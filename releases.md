# Releases

## v0.3.9

- Require `bake` 0.20 or later, instead of any 0.x version, so that Cargo resolves this crate to the same `bake` as a task package that requires `bake` 0.20.2. In a workspace whose `rust-version` is below the 1.88 that `bake` 0.20.2 declares, Cargo resolved the old requirement to `bake` 0.19.2, which declares none, and the task package failed to compile against two `bake` versions.

## v0.3.8

- Declare Rust 1.88 as the minimum supported version in `rust-version`, as required by the let-chains in this crate and in `bake` 0.20.

## v0.3.7

- Add a `license` job to the canonical `test.yml`, which runs `license:check` on the pull request's head commit with its full history, and require it in the `test-result` aggregate, so a missing or stale license header fails the pull request that introduces it rather than changing sources in a release.

## v0.3.6

- Add a `minimum-rust` job to the canonical `test.yml`, which reads `rust-version` from the root `Cargo.toml` and checks every publishable package with that toolchain, and require it in the `test-result` aggregate.

## v0.3.5

- Let a package exclude specific files from the coverage gate under `[[package.metadata.bake.test.coverage.exclude]]`, each with a required `reason`. The task rejects missing reasons, missing files, and duplicate entries, and always lists each excluded file with its reason and size in regions and lines.

## v0.3.4

- Update package metadata and repository links for the `base-rs` organization.

## v0.3.3

- Publish as `bake-test` from `core-rs/bake-test`; update imports to `bake_test`.
- Require a `Release: vVERSION` Git trailer on the final release commit; version changes alone no longer trigger publication.

## v0.3.2

- Use Cargo-selected dependency versions and generated task links in setup instructions.

## v0.3.1

- Adopt `socketry-project` 0.3.7 for shared project tasks and Markdown normalization.
- Require the aggregate test and coverage result for pull request merges.
- Document the canonical aggregate test gate and current coverage dependency.

## v0.3.0

- Enforce complete source-region coverage, excluding regions inside invariant-only `unreachable!()` calls.

## v0.2.3

- Exclude uncovered, single-line `unreachable!("reason")` expressions from measured coverage automatically.

## v0.2.2

- Declare compatibility with the Bake 0.x API so task libraries can share one task registry when upgrading to crate-derived task namespaces.

- Document how to install and discover dependency context and skills.

- Assert external test runs refresh each checkout's Cargo lockfile.

## v0.2.1

- Use the shared `socketry-project` Releasing skill for the standard release process and remove references to the duplicate Bake Cargo publishing context.

## v0.2.0

- Add a Bake task for 100% line coverage with missing-line output.

## v0.1.2

- Update downstream lockfiles after applying local workspace patches.

## v0.1.1

- Fix the Bake task name so `cargo bake test` invokes the workspace test task.

## v0.1.0

- Add standard Rust test and downstream compatibility tasks for Bake.
