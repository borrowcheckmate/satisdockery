# Contributing

## Setup

You need a Rust toolchain (see `rust-version` in `Cargo.toml`) and [pre-commit](https://pre-commit.com).

```sh
pre-commit install
```

This installs the git hooks for both commits and commit messages.

## Checks

Pre-commit runs formatting, clippy, the file length rule, and commit message validation. Tests run in CI.

```sh
pre-commit run --all-files   # lint everything
cargo test --locked          # run the tests locally
cargo deny check             # licenses and advisories
```

Clippy is configured strictly in `Cargo.toml` and `clippy.toml`. Fix the warning instead of silencing it. If an `allow` is truly needed it must carry a `reason`.

Dependencies, GitHub Actions and the base images are updated weekly by Dependabot. CI builds the container image for both architectures and starts it as a smoke test.

## Code style

- Code explains itself through names and small functions. Add a comment only for a reason that the code cannot show.
- Keep files under 250 lines. Split larger files into submodules, see [ARCHITECTURE.md](ARCHITECTURE.md).
- Dependencies are pinned to an exact version (`=x.y.z`).
- No `unwrap`, `expect`, `panic!` or indexing in non-test code.

## Commits

Commits follow [Conventional Commits](https://www.conventionalcommits.org), because the release notes and version bumps are derived from them: `feat:` bumps the minor version, `fix:` the patch version, and `feat!:` or a `BREAKING CHANGE:` footer the major version.

Commits contain no attribution lines or trailers.

## Releases

Releases are automated by [release-plz](https://release-plz.dev):

1. Every push to `main` updates a release pull request with the version bump and changelog.
2. Merging that pull request tags the release and creates a GitHub release.
3. The same workflow builds the multi-architecture image and pushes it to `ghcr.io`.

The release workflow needs a repository secret named `RELEASE_PLZ_TOKEN`: a fine-grained token with `contents` and `pull requests` write access. A token is used, not the default `GITHUB_TOKEN`, so that CI runs on the release pull request.
