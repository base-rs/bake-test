// Released under the MIT License.
// Copyright, 2026, by Samuel Williams.

use bake::{Registry, Result};

fn main() -> Result<()> {
    Registry::discover()?.run()
}

// Refresh release files through the directly linked task providers.
#[bake::task(name = "cargo:after_version_bump")]
fn after_version_bump(context: &mut bake::Context, version: String) -> Result<()> {
    context.call("license:update", &[])?;
    context.call("releases:update", &[&format!("v{version}")])?;
    context.call("readme:update", &[])?;
    context.call(
        "markdown:normalize",
        &[
            "context/testing.md",
            "license.md",
            "readme.md",
            "releases.md",
        ],
    )?;
    Ok(())
}

#[cfg(test)]
#[bake::task(name = "test")]
fn duplicate_test_task(#[bake(context)] _context: &mut bake::Context) -> Result<bake::Value> {
    Ok(bake::Value::Null)
}

#[cfg(test)]
mod tests {
    use super::after_version_bump;
    use bake::{Arguments, Context, Parameter, Registry, Result, Task, Value};
    use std::fs;

    #[test]
    fn reports_duplicate_discovered_tasks() {
        let error = super::main().expect_err("duplicate task registration should fail");
        assert!(error.to_string().contains("duplicate task \"test\""));

        let mut context = Registry::new().context(".");
        super::duplicate_test_task(&mut context).expect("duplicate task fixture should run");
    }

    fn license_update(context: &mut Context, _arguments: &Arguments) -> Result<Value> {
        fs::write(context.root().join("license-called"), "yes").expect("write license marker");
        Ok(Value::Null)
    }

    fn releases_update(context: &mut Context, arguments: &Arguments) -> Result<Value> {
        let version = arguments
            .required::<String>("version")
            .expect("version is supplied to the release task");
        fs::write(context.root().join("release-version"), version)
            .expect("write release version marker");
        Ok(Value::Null)
    }

    fn readme_update(context: &mut Context, _arguments: &Arguments) -> Result<Value> {
        fs::write(context.root().join("readme-called"), "yes").expect("write Readme marker");
        Ok(Value::Null)
    }

    fn normalize_markdown(context: &mut Context, arguments: &Arguments) -> Result<Value> {
        let paths = arguments
            .repeated::<String>("paths")
            .expect("valid Markdown paths");
        assert!(context.root().join("readme-called").exists());
        fs::write(context.root().join("markdown-paths"), paths.join("\n"))
            .expect("write Markdown paths");
        Ok(Value::Null)
    }

    #[test]
    fn updates_release_files_in_order_with_the_bumped_version() {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system clock is after the Unix epoch")
            .as_nanos();
        let root =
            std::env::temp_dir().join(format!("bake-test-bake-{}-{unique}", std::process::id()));
        fs::create_dir_all(root.join("context")).expect("create temporary Bake project");
        fs::write(root.join("context/testing.md"), "# Testing\n").unwrap();
        for task_count in 0..=4 {
            let tasks = [
                Task::new("license:update", "", Vec::new(), license_update),
                Task::new(
                    "releases:update",
                    "",
                    vec![Parameter::new::<String>("version")],
                    releases_update,
                ),
                Task::new("readme:update", "", Vec::new(), readme_update),
                Task::new(
                    "markdown:normalize",
                    "",
                    vec![Parameter::new::<String>("paths").variadic()],
                    normalize_markdown,
                ),
            ];
            let missing = tasks.get(task_count).map(|task| task.name().to_owned());
            let mut registry = Registry::new();
            for task in tasks.into_iter().take(task_count) {
                registry.register(task).expect("register hook task");
            }
            let result = after_version_bump(&mut registry.context(&root), "1.2.3".to_owned());
            if let Some(missing) = missing {
                assert!(result.unwrap_err().to_string().contains(&missing));
            } else {
                result.expect("run release update tasks");
            }
        }

        assert_eq!(
            fs::read_to_string(root.join("license-called")).unwrap(),
            "yes"
        );
        assert_eq!(
            fs::read_to_string(root.join("release-version")).unwrap(),
            "v1.2.3"
        );
        assert_eq!(
            fs::read_to_string(root.join("readme-called")).unwrap(),
            "yes"
        );
        assert_eq!(
            fs::read_to_string(root.join("markdown-paths")).unwrap(),
            "context/testing.md\nlicense.md\nreadme.md\nreleases.md"
        );
        fs::remove_dir_all(root).expect("remove temporary Bake project");
    }
}

#[path = "bake_generated_tasks/mod.rs"]
mod bake_generated_tasks;
