# Agent instructions

Rules for automated coding agents working in this repository. Read [CONTRIBUTING.md](CONTRIBUTING.md) and [ARCHITECTURE.md](ARCHITECTURE.md) first.

## Before finishing

Run, and fix everything they report:

```sh
cargo fmt --all
cargo clippy --all-targets --locked -- -D warnings
cargo test --locked
```

## Rules

- Do not weaken lints. Do not add `#[allow]` without a `reason`.
- Do not add documentation files besides those already in the repository root.
- Keep files under 250 lines by splitting them into submodules.
- Pin new dependencies to an exact version.
- Write commit messages as Conventional Commits.

## Privacy

- Never add names, emails, handles, hostnames or local paths of any person or machine to code, docs, configuration or commit messages.
- Never add attribution of any kind: no `Co-Authored-By` trailers, no "generated with" lines, no tool advertising, in commits, pull requests or files.
- Commit only with the repository's configured identity.
