# SatisDockery

[![CI](https://github.com/borrowcheckmate/satisdockery/actions/workflows/ci.yml/badge.svg)](https://github.com/borrowcheckmate/satisdockery/actions/workflows/ci.yml)
[![Release](https://img.shields.io/github/v/release/borrowcheckmate/satisdockery?display_name=tag)](https://github.com/borrowcheckmate/satisdockery/releases)
[![Container image](https://img.shields.io/badge/ghcr.io-satisdockery-2496ED?logo=docker&logoColor=white)](https://github.com/borrowcheckmate/satisdockery/pkgs/container/satisdockery)
[![Platforms](https://img.shields.io/badge/platforms-linux%2Famd64%20%7C%20linux%2Farm64-informational)](#features)
[![License: EUPL-1.2](https://img.shields.io/badge/license-EUPL--1.2-blue)](LICENSE)

[![Rust 1.88+](https://img.shields.io/badge/rust-1.88%2B-orange?logo=rust)](Cargo.toml)
[![Clippy: pedantic](https://img.shields.io/badge/clippy-pedantic%20%2B%20nursery-success)](Cargo.toml)
[![Conventional Commits](https://img.shields.io/badge/commits-conventional-fe5196?logo=conventionalcommits&logoColor=white)](https://www.conventionalcommits.org)
[![Releases by release-plz](https://img.shields.io/badge/releases-release--plz-8A2BE2)](https://release-plz.dev)
[![Last commit](https://img.shields.io/github/last-commit/borrowcheckmate/satisdockery)](https://github.com/borrowcheckmate/satisdockery/commits/main)

A [Satisfactory](https://www.satisfactorygame.com) dedicated server in Docker that installs itself and its mods.

On every start it downloads or updates the server, picks mod versions that work with the installed game, installs them, and launches the server. A small Rust program does the work, so the image needs no scripting.

## Features

- **Runs on `amd64` and `arm64`.** On `arm64` the server runs through [box64](https://github.com/ptitSeb/box64), so a Raspberry Pi class machine or an ARM VPS works.
- **Mods from [ficsit.app](https://ficsit.app).** List the mods you want and a version constraint. Dependencies are resolved for you, and only versions marked as working on the installed game build are installed.
- **Always current.** The server is updated through DepotDownloader at startup, unless you turn that off.
- **Verified downloads.** DepotDownloader and box64 are pinned and checked against SHA-256 checksums before they are used.
- **Configured by environment variables.** No wrapper script and no config files to mount.
- **Hardened by default.** Runs as a non-root user and rejects mod names that could write outside the mods directory.

## Quick start

```yaml
services:
  satisfactory:
    image: ghcr.io/borrowcheckmate/satisdockery:latest
    restart: unless-stopped
    ports:
      - "7777:7777/udp"
      - "8888:8888/tcp"
    environment:
      SATISDOCKERY_PLAYERS: "8"
      SATISDOCKERY_MODS: "RefinedPower:>=4.0.44"
    volumes:
      - satisfactory:/home/satisdockery/satisfactory

volumes:
  satisfactory:
```

The first start downloads the game, which takes a while. Later starts only fetch updates.

## Mods

Set `SATISDOCKERY_MODS` (or `--mods`) to a comma separated list of `Name:constraint` entries, where the name is the mod reference shown on ficsit.app and the constraint is a [semver](https://docs.rs/semver/latest/semver/enum.Op.html) requirement:

```
SATISDOCKERY_MODS=RefinedPower:>=4.0.44,SecondMod:^1.2
```

The Satisfactory Mod Loader and everything your mods depend on is added automatically. For each mod the highest version that satisfies all constraints and works on the installed game is installed.

## Configuration

Every option is available as a command line flag and as an environment variable named `SATISDOCKERY_` followed by the flag in upper case with underscores, for example `--max-tickrate` is `SATISDOCKERY_MAX_TICKRATE`. Switches take `true` or `false`.

| Option | Default | Description |
| --- | --- | --- |
| `--path` | `/home/satisdockery/satisfactory` in the image | Satisfactory installation directory |
| `--mods` | none | Comma separated `Name:constraint` list |
| `--skip-update` | off | Do not download or update the server |
| `--skip-run` | off | Prepare the installation but do not start the server |
| `--players` | `4` | Maximum number of players |
| `--game-port` | `7777` | Game port (UDP) |
| `--messaging-port` | `8888` | Messaging port (TCP) |
| `--max-autosaves` | `5` | Number of rotating autosaves |
| `--max-objects` | `2162688` | Maximum number of objects in the world |
| `--max-tickrate` | `30` | Maximum tick rate |
| `--max-timeout` | `10` | Seconds until an unresponsive player is kicked |
| `--enable-serverstreaming` | off | Only stream part of the world to clients |
| `--disable-seasonalevents` | off | Disable seasonal events |
| `--additional-dedicatedserver-args` | none | Extra Unreal arguments, comma separated |

Run `satisdockery --help` for the authoritative list.

## Migrating from Wolveix's image

[Wolveix's image](https://github.com/wolveix/satisfactory-server) works great. This project adds `arm64` support and automatic mod installation, and Wolveix's work made it clear how the server setup works.

To switch, replace the image with `ghcr.io/borrowcheckmate/satisdockery:latest` and translate your environment variables to the `SATISDOCKERY_` names above.

## Building from source

```sh
cargo build --release
./target/release/satisdockery --help
```

Or build the container image yourself:

```sh
docker build -t satisdockery .
```

## Project documentation

- [ARCHITECTURE.md](ARCHITECTURE.md) explains how the code is organised.
- [CONTRIBUTING.md](CONTRIBUTING.md) explains how to develop, test and release.
- [AGENTS.md](AGENTS.md) holds the rules for automated coding agents.

## License

[EUPL-1.2](LICENSE)
