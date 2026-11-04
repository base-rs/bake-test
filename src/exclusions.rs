// Released under the MIT License.
// Copyright, 2026, by Samuel Williams.

//! Files that packages exclude from the coverage gate.
//!
//! Each workspace package may list files in its own `Cargo.toml`:
//!
//! ```toml
//! [[package.metadata.bake.test.coverage.exclude]]
//! path = "src/xr.rs"
//! reason = "Raw OpenXR runtime calls; needs a headset runtime."
//! ```
//!
//! Entries name exact files relative to the package, so each file carries its
//! own reason and size in the coverage summary, and a renamed or deleted file
//! fails the task instead of leaving a stale entry behind.

use bake::{Context, Error, Result};
use serde_json::Value;
use std::collections::HashSet;
use std::path::{Component, Path, PathBuf};

use crate::coverage::resolve_source_path;
use crate::metadata::{read_metadata, required_string};

const ENTRY_KEYS: [&str; 2] = ["path", "reason"];

/// A source file whose regions are reported but not counted towards the coverage gate.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Exclusion {
    /// The canonical path, matched against source paths in the coverage report.
    pub path: PathBuf,
    /// The path relative to the workspace root, for messages.
    pub display: String,
    pub reason: String,
}

pub(crate) fn read(context: &Context) -> Result<Vec<Exclusion>> {
    parse(&read_metadata(context)?)
}

fn parse(metadata: &Value) -> Result<Vec<Exclusion>> {
    let workspace_root = metadata
        .get("workspace_root")
        .and_then(Value::as_str)
        .map(PathBuf::from)
        .ok_or_else(|| Error::new("Cargo metadata did not contain a workspace root"))?;
    let members = metadata
        .get("workspace_members")
        .and_then(Value::as_array)
        .ok_or_else(|| Error::new("Cargo metadata did not contain workspace members"))?;
    let packages = metadata
        .get("packages")
        .and_then(Value::as_array)
        .ok_or_else(|| Error::new("Cargo metadata did not contain packages"))?;

    let mut exclusions = Vec::new();
    let mut seen = HashSet::new();
    for package in packages {
        let identifier = required_string(package, "id", "Cargo package")?;
        if !members
            .iter()
            .any(|member| member.as_str() == Some(identifier.as_str()))
        {
            continue;
        }

        let Some(coverage) = package
            .get("metadata")
            .and_then(|value| value.get("bake"))
            .and_then(|value| value.get("test"))
            .and_then(|value| value.get("coverage"))
        else {
            continue;
        };

        let name = required_string(package, "name", "Cargo package")?;
        let manifest_path =
            PathBuf::from(required_string(package, "manifest_path", "Cargo package")?);
        let package_directory = manifest_path.parent().ok_or_else(|| {
            Error::new(format!(
                "package {name} has a manifest path without a parent directory"
            ))
        })?;
        let entries = parse_entries(coverage, &name)?;

        for (path, reason) in entries {
            let exclusion =
                resolve_entry(&workspace_root, package_directory, &name, &path, reason)?;
            if !seen.insert(exclusion.path.clone()) {
                return Err(Error::new(format!(
                    "package {name} excludes {} from coverage more than once; list each file once",
                    exclusion.display
                )));
            }
            exclusions.push(exclusion);
        }
    }

    exclusions.sort_by(|left, right| left.display.cmp(&right.display));
    Ok(exclusions)
}

fn parse_entries(coverage: &Value, package: &str) -> Result<Vec<(String, String)>> {
    let Some(exclude) = coverage.get("exclude") else {
        return Ok(Vec::new());
    };
    let entries = exclude.as_array().ok_or_else(|| {
        Error::new(format!(
            "package {package}: [package.metadata.bake.test.coverage] exclude must be an array of tables; use [[package.metadata.bake.test.coverage.exclude]]"
        ))
    })?;

    entries
        .iter()
        .map(|entry| parse_entry(entry, package))
        .collect()
}

fn parse_entry(entry: &Value, package: &str) -> Result<(String, String)> {
    let table = entry.as_object().ok_or_else(|| {
        Error::new(format!(
            "package {package}: each coverage exclusion must be a table with path and reason"
        ))
    })?;
    if let Some(key) = table.keys().find(|key| !ENTRY_KEYS.contains(&key.as_str())) {
        return Err(Error::new(format!(
            "package {package}: coverage exclusion has unknown key {key:?}; expected path and reason"
        )));
    }

    let path = table
        .get("path")
        .and_then(Value::as_str)
        .filter(|path| !path.is_empty())
        .ok_or_else(|| {
            Error::new(format!(
                "package {package}: each coverage exclusion needs a path, relative to the package"
            ))
        })?;
    let reason = table
        .get("reason")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|reason| !reason.is_empty())
        .ok_or_else(|| {
            Error::new(format!(
                "package {package}: the coverage exclusion for {path} needs a nonempty reason explaining why it cannot be tested"
            ))
        })?;

    Ok((path.to_owned(), reason.to_owned()))
}

fn resolve_entry(
    workspace_root: &Path,
    package_directory: &Path,
    package: &str,
    path: &str,
    reason: String,
) -> Result<Exclusion> {
    let relative_path = Path::new(path);
    if !relative_path
        .components()
        .all(|component| matches!(component, Component::Normal(_)))
    {
        return Err(Error::new(format!(
            "package {package}: coverage exclusion {path} must be a relative path inside the package, without . or .. components"
        )));
    }

    let package_path = package_directory.join(relative_path);
    let display_path = package_path
        .strip_prefix(workspace_root)
        .unwrap_or(&package_path);
    let display = display_path
        .components()
        .collect::<PathBuf>()
        .to_string_lossy()
        .replace('\\', "/");
    let canonical_path = resolve_source_path(&package_path, package_directory);

    if !canonical_path.is_file() {
        let problem = if canonical_path.exists() {
            "is not a file; list each excluded file"
        } else if path.contains(['*', '?', '[', '{']) {
            "does not exist; glob patterns are not supported, so list each excluded file"
        } else {
            "does not exist; remove or update the stale entry"
        };
        return Err(Error::new(format!(
            "package {package}: coverage exclusion {display} {problem}"
        )));
    }

    Ok(Exclusion {
        path: canonical_path,
        display,
        reason,
    })
}

#[cfg(test)]
#[path = "exclusions/tests.rs"]
mod tests;
