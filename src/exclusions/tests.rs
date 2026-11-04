// Released under the MIT License.
// Copyright, 2026, by Samuel Williams.

use super::{Exclusion, parse, read};
use crate::test_support::{Environment, TemporaryDirectory};
use bake::Registry;
use serde_json::{Value, json};
use std::fs;
use std::path::Path;

fn write_file(root: &Path, path: &str) {
    let path = root.join(path);
    fs::create_dir_all(path.parent().expect("file parent")).expect("create file directory");
    fs::write(path, "fn device() {}\n").expect("write file");
}

fn package(root: &Path, id: &str, directory: &str, coverage: Option<Value>) -> Value {
    let mut package = json!({
        "id": id,
        "name": id,
        "manifest_path": root.join(directory).join("Cargo.toml"),
    });
    if let Some(coverage) = coverage {
        package["metadata"] = json!({"bake": {"test": {"coverage": coverage}}});
    }
    package
}

fn workspace(root: &Path, members: &[&str], packages: Vec<Value>) -> Value {
    json!({
        "workspace_root": root,
        "workspace_members": members,
        "packages": packages,
    })
}

fn exclusions(entries: Value) -> Value {
    json!({"exclude": entries})
}

/// Parses one member package, `headset`, at `crates/headset`, with the given coverage metadata.
fn parse_headset(root: &Path, coverage: Value) -> bake::Result<Vec<Exclusion>> {
    parse(&workspace(
        root,
        &["headset"],
        vec![package(root, "headset", "crates/headset", Some(coverage))],
    ))
}

fn headset_error(root: &Path, entries: Value) -> String {
    parse_headset(root, exclusions(entries))
        .expect_err("invalid coverage exclusion")
        .to_string()
}

#[test]
fn reads_exclusions_from_each_workspace_member() {
    let directory = TemporaryDirectory::new();
    let root = directory.path();
    write_file(root, "crates/headset/src/xr.rs");
    write_file(root, "crates/headset/src/window.rs");
    write_file(root, "src/audio.rs");

    let metadata = workspace(
        root,
        &["headset", "root", "plain", "empty"],
        vec![
            package(
                root,
                "headset",
                "crates/headset",
                Some(exclusions(json!([
                    {"path": "src/xr.rs", "reason": " Raw OpenXR runtime calls. "},
                    {"path": "src/window.rs", "reason": "Window system event handlers."}
                ]))),
            ),
            package(
                root,
                "root",
                "",
                Some(exclusions(json!([
                    {"path": "src/audio.rs", "reason": "Audio device calls."}
                ]))),
            ),
            package(root, "plain", "crates/plain", None),
            package(root, "empty", "crates/empty", Some(json!({}))),
            // Only workspace members are read, so this missing file is not validated.
            package(
                root,
                "dependency",
                "dependency",
                Some(exclusions(json!([
                    {"path": "src/missing.rs", "reason": "Not a member."}
                ]))),
            ),
        ],
    );

    let exclusions = parse(&metadata).expect("valid coverage exclusions");
    assert_eq!(
        exclusions
            .iter()
            .map(|exclusion| (exclusion.display.as_str(), exclusion.reason.as_str()))
            .collect::<Vec<_>>(),
        [
            (
                "crates/headset/src/window.rs",
                "Window system event handlers."
            ),
            ("crates/headset/src/xr.rs", "Raw OpenXR runtime calls."),
            ("src/audio.rs", "Audio device calls."),
        ]
    );
    assert_eq!(
        exclusions[1].path,
        fs::canonicalize(root.join("crates/headset/src/xr.rs")).expect("canonical path")
    );
}

#[test]
fn displays_paths_outside_the_workspace_root_in_full() {
    let directory = TemporaryDirectory::new();
    let root = directory.path();
    write_file(root, "outside/src/xr.rs");

    let exclusions = parse(&workspace(
        &root.join("workspace"),
        &["outside"],
        vec![package(
            root,
            "outside",
            "outside",
            Some(exclusions(
                json!([{"path": "src/xr.rs", "reason": "Device calls."}]),
            )),
        )],
    ))
    .expect("member outside the workspace root");

    assert_eq!(
        Path::new(&exclusions[0].display),
        root.join("outside/src/xr.rs")
    );
}

#[test]
fn requires_a_reason_for_each_exclusion() {
    let directory = TemporaryDirectory::new();
    let root = directory.path();
    write_file(root, "crates/headset/src/xr.rs");

    for entries in [
        json!([{"path": "src/xr.rs"}]),
        json!([{"path": "src/xr.rs", "reason": ""}]),
        json!([{"path": "src/xr.rs", "reason": "  \n"}]),
        json!([{"path": "src/xr.rs", "reason": true}]),
    ] {
        assert_eq!(
            headset_error(root, entries),
            "package headset: the coverage exclusion for src/xr.rs needs a nonempty reason explaining why it cannot be tested"
        );
    }
}

#[test]
fn requires_a_path_for_each_exclusion() {
    let directory = TemporaryDirectory::new();

    for entries in [
        json!([{"reason": "Device calls."}]),
        json!([{"path": "", "reason": "Device calls."}]),
        json!([{"path": 1, "reason": "Device calls."}]),
    ] {
        assert_eq!(
            headset_error(directory.path(), entries),
            "package headset: each coverage exclusion needs a path, relative to the package"
        );
    }
}

#[test]
fn rejects_malformed_exclusion_lists_and_entries() {
    let directory = TemporaryDirectory::new();
    let root = directory.path();

    assert!(
        parse_headset(root, json!({"exclude": "src/xr.rs"}))
            .expect_err("exclude must be an array")
            .to_string()
            .contains("exclude must be an array of tables")
    );
    assert_eq!(
        headset_error(root, json!(["src/xr.rs"])),
        "package headset: each coverage exclusion must be a table with path and reason"
    );
    assert_eq!(
        headset_error(
            root,
            json!([{"path": "src/xr.rs", "reason": "Device calls.", "lines": "1-10"}])
        ),
        "package headset: coverage exclusion has unknown key \"lines\"; expected path and reason"
    );
}

#[test]
fn rejects_paths_outside_the_package() {
    let directory = TemporaryDirectory::new();
    let root = directory.path();
    write_file(root, "crates/headset/src/xr.rs");
    write_file(root, "crates/other/src/xr.rs");
    let absolute = root.join("crates/headset/src/xr.rs");

    for path in [
        absolute.to_str().expect("UTF-8 path"),
        "../other/src/xr.rs",
        "./src/xr.rs",
        "src/../src/xr.rs",
    ] {
        assert_eq!(
            headset_error(root, json!([{"path": path, "reason": "Device calls."}])),
            format!(
                "package headset: coverage exclusion {path} must be a relative path inside the package, without . or .. components"
            )
        );
    }
}

#[test]
fn rejects_missing_files_directories_and_globs() {
    let directory = TemporaryDirectory::new();
    let root = directory.path();
    write_file(root, "crates/headset/src/xr.rs");

    assert_eq!(
        headset_error(
            root,
            json!([{"path": "src/removed.rs", "reason": "Device calls."}])
        ),
        "package headset: coverage exclusion crates/headset/src/removed.rs does not exist; remove or update the stale entry"
    );
    assert_eq!(
        headset_error(root, json!([{"path": "src", "reason": "Device calls."}])),
        "package headset: coverage exclusion crates/headset/src is not a file; list each excluded file"
    );
    for path in ["src/*.rs", "src/x?.rs", "src/[x]r.rs", "src/{xr}.rs"] {
        assert_eq!(
            headset_error(root, json!([{"path": path, "reason": "Device calls."}])),
            format!(
                "package headset: coverage exclusion crates/headset/{path} does not exist; glob patterns are not supported, so list each excluded file"
            )
        );
    }
}

#[test]
fn rejects_duplicate_exclusions() {
    let directory = TemporaryDirectory::new();
    let root = directory.path();
    write_file(root, "crates/headset/src/xr.rs");

    assert_eq!(
        headset_error(
            root,
            json!([
                {"path": "src/xr.rs", "reason": "Device calls."},
                {"path": "src//xr.rs", "reason": "The same file again."}
            ])
        ),
        "package headset excludes crates/headset/src/xr.rs from coverage more than once; list each file once"
    );
}

#[test]
fn rejects_invalid_workspace_metadata() {
    let directory = TemporaryDirectory::new();
    let root = directory.path();
    let headset = package(
        root,
        "headset",
        "crates/headset",
        Some(exclusions(json!([]))),
    );

    for (metadata, message) in [
        (
            json!({"workspace_members": [], "packages": []}),
            "Cargo metadata did not contain a workspace root",
        ),
        (
            json!({"workspace_root": root, "packages": []}),
            "Cargo metadata did not contain workspace members",
        ),
        (
            json!({"workspace_root": root, "workspace_members": []}),
            "Cargo metadata did not contain packages",
        ),
        (
            workspace(root, &[], vec![json!({"name": "unnamed"})]),
            "Cargo package must contain a nonempty id",
        ),
        (
            workspace(
                root,
                &["headset"],
                vec![json!({"id": "headset", "metadata": headset["metadata"]})],
            ),
            "Cargo package must contain a nonempty name",
        ),
        (
            workspace(
                root,
                &["headset"],
                vec![json!({"id": "headset", "name": "headset", "metadata": headset["metadata"]})],
            ),
            "Cargo package must contain a nonempty manifest_path",
        ),
        (
            workspace(
                root,
                &["headset"],
                vec![json!({
                    "id": "headset",
                    "name": "headset",
                    "manifest_path": "/",
                    "metadata": headset["metadata"],
                })],
            ),
            "package headset has a manifest path without a parent directory",
        ),
    ] {
        assert_eq!(
            parse(&metadata)
                .expect_err("invalid workspace metadata")
                .to_string(),
            message
        );
    }
}

#[cfg(unix)]
#[test]
fn reads_exclusions_through_cargo_metadata() {
    let environment = Environment::new();
    let directory = TemporaryDirectory::new();
    let root = directory.path();
    write_file(root, "src/xr.rs");
    let metadata_path = root.join("workspace-metadata.json");
    fs::write(
        &metadata_path,
        workspace(
            root,
            &["root"],
            vec![package(
                root,
                "root",
                "",
                Some(exclusions(
                    json!([{"path": "src/xr.rs", "reason": "Device calls."}]),
                )),
            )],
        )
        .to_string(),
    )
    .expect("write workspace metadata");
    environment.set(
        "CARGO",
        crate::test_support::install_fake_cargo(directory.path()),
    );
    environment.set("BAKE_TEST_CARGO_LOG", root.join("cargo.log"));
    environment.set("BAKE_TEST_WORKSPACE_METADATA", &metadata_path);
    environment.set("BAKE_TEST_METADATA_EXIT", "0");

    let context = Registry::new().context(root);
    let exclusions = read(&context).expect("read exclusions through Cargo");
    assert_eq!(exclusions.len(), 1);
    assert_eq!(exclusions[0].display, "src/xr.rs");

    environment.set("BAKE_TEST_METADATA_EXIT", "3");
    assert!(
        read(&context)
            .expect_err("Cargo metadata failure")
            .to_string()
            .contains("cargo metadata failed")
    );
}
