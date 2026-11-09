# `bake-test`

Reusable Bake tasks for running Rust tests and testing downstream repositories against local Cargo workspace packages.

## Motivation

Rust provides a good test runner through Cargo, but it does not provide a shared way to prepare project resources or exercise selected downstream crates. This package gives Rust projects a consistent `bake test`, `bake test:coverage`, and `bake test:external` interface while keeping Cargo and cargo-llvm-cov as the test runners.

## Usage

Add the task crate to the private `bake/` package:

```sh
cargo bake --regenerate
cargo add --manifest-path bake/Cargo.toml bake-test
cargo bake --regenerate
```

Then run the workspace tests:

```sh
cargo bake test
cargo bake test:coverage
cargo bake test:external
```

The coverage task runs documentation tests and requires 100% measured source-region coverage for the selected workspace test configuration. Regions inside `unreachable!()` calls are excluded automatically. It reports uncovered source locations when the gate fails. Install `cargo-llvm-cov` first; see the [Rust Testing guide](context/testing.md) for feature selection and architecture notes. Use `--package name` to limit a workspace run to one package.

As a last resort, a package can exclude from the gate a file whose code cannot run in CI, such as raw device or platform calls. List each file in the package's `Cargo.toml` with a reason:

```toml
[[package.metadata.bake.test.coverage.exclude]]
path = "src/xr.rs"
reason = "Raw OpenXR runtime calls; needs a headset runtime."
```

The coverage summary always lists each excluded file with its reason and size. See [Excluding files](context/testing.md#excluding-files) for the rules.

Configure downstream repositories in the root `Cargo.toml`:

```toml
[[workspace.metadata.bake.test.external]]
repository = "https://github.com/socketry/downstream-project"
branch = "main"
```

See the [Rust Testing guide](context/testing.md) for the `test:before` hook, checkout reuse, Cargo patching, and canonical GitHub Actions workflows.

## Releasing

Prepare a release with `cargo bake cargo:release:patch` (or `minor`, `major`, or `bump --version X.Y.Z`) from a clean checkout of `main`; it bumps the version, commits with the `Release: vVERSION` trailer, validates the candidate, and opens the pull request. After review and merge, GitHub Actions publishes the release when the configured `crates-io` environment approves it. Follow the shared [Releasing skill](https://github.com/base-rs/base-project/blob/main/context/releasing.md) for the standard process.

## Releases

<!-- bake-readme:releases:start -->

See [releases.md](releases.md) for the full release history.

### v0.3.9

- Require `bake` 0.20 or later, instead of any 0.x version, so that Cargo resolves this crate to the same `bake` as a task package that requires `bake` 0.20.2. In a workspace whose `rust-version` is below the 1.88 that `bake` 0.20.2 declares, Cargo resolved the old requirement to `bake` 0.19.2, which declares none, and the task package failed to compile against two `bake` versions.

### v0.3.8

- Declare Rust 1.88 as the minimum supported version in `rust-version`, as required by the let-chains in this crate and in `bake` 0.20.

### v0.3.7

- Add a `license` job to the canonical `test.yml`, which runs `license:check` on the pull request's head commit with its full history, and require it in the `test-result` aggregate, so a missing or stale license header fails the pull request that introduces it rather than changing sources in a release.

<!-- bake-readme:releases:end -->

## See Also

- [Bake](https://github.com/base-rs/bake)
- [Bake Cargo](https://github.com/base-rs/bake-cargo)

## Contributing

Please open an issue or pull request on [GitHub](https://github.com/base-rs/bake-test).

### Agent Context

Run `cargo bake agent:context:install` to install shared context and skills. Read `.agents/context/index.md` to find relevant guides, follow `agents.md` if present, and apply skills under `.agents/skills/`. The installer preserves repository-owned `agents.md`; it does not create or regenerate that file.
