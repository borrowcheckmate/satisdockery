# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0](https://github.com/borrowcheckmate/satisdockery/releases/tag/v0.1.0) - 2026-10-28

### Added

- configure every option through SATISDOCKERY_ environment variables
- start the server with configurable Unreal arguments
- resolve and install mods from ficsit.app
- download the dedicated server with a verified DepotDownloader
- read the installed game version

### Fixed

- start the server from the container entrypoint

### Other

- add status badges to the readme
- rewrite the readme with features, quick start and badges
- rewrite the readme with features, quick start and badges
- allow starting the release workflow manually
- only release when the release pull request is merged
- *(deps)* bump the cargo group with 7 updates
- fix the quoting of the multi-arch image merge step
- document environment variables and container usage
- update dependencies, actions and base images weekly
- smoke test the container image on both architectures
- drop duct and tracing-subscriber and trim default features
- build each image platform on a native runner
- add architecture, contributing and agent guidelines
- add readme
- automate releases and image publishing with release-plz
- lint, test, audit dependencies and build the image
- add multi-architecture container image
- cover startup against a fake installation
- add lint, formatting and supply chain configuration
- Initial commit
