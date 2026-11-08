Use `cargo bake cargo:release:patch` (or `minor`, `major`, or
`bump --version X.Y.Z`) from a clean checkout of `main` to prepare a release.
It bumps the version, commits it with the `Release: vVERSION` trailer,
validates the candidate, and opens the release pull request. Review the
generated changes before merging. The merged release pull request is published
by GitHub Actions after the configured `crates-io` environment approves it.
