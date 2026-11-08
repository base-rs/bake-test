// Released under the MIT License.
// Copyright, 2026, by Samuel Williams.

use bake::{Registry, Result};

fn main() -> Result<()> {
    Registry::discover()?.run()
}

#[cfg(test)]
#[bake::task(name = "test")]
fn duplicate_test_task(#[bake(context)] _context: &mut bake::Context) -> Result<bake::Value> {
    Ok(bake::Value::Null)
}

#[cfg(test)]
mod tests {
    use bake::Registry;

    #[test]
    fn reports_duplicate_discovered_tasks() {
        let error = super::main().expect_err("duplicate task registration should fail");
        assert!(error.to_string().contains("duplicate task \"test\""));

        let mut context = Registry::new().context(".");
        super::duplicate_test_task(&mut context).expect("duplicate task fixture should run");
    }
}

#[path = "bake_generated_tasks/mod.rs"]
mod bake_generated_tasks;
