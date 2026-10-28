# Architecture

SatisDockery is a library crate (`src/lib.rs`) with a thin binary (`src/main.rs`) on top. The binary only initialises logging, parses arguments and turns the result of `satisdockery::run` into an exit code.

## Flow

`run` in `src/lib.rs` executes the steps in order:

1. **Download the server** (`depot`), unless `--skip-update` is set.
2. **Read the game version** (`game`) from the installed server.
3. **Resolve mods** (`mods`) to concrete packages that are compatible with that version.
4. **Install mods** (`mods`) into the installation.
5. **Run the server** (`server`), unless `--skip-run` is set.

## Modules

| Module | Responsibility |
| --- | --- |
| `cli` | Command line arguments |
| `error` | The crate-wide `Error` and `Result` |
| `game` | The installed game version and the layout of the installation |
| `depot` | Installing a checksum-verified DepotDownloader and running it |
| `mods` | Mod resolution and installation |
| `server` | Building the Unreal arguments and starting the server process |

### `depot`

DepotDownloader releases are pinned per architecture together with the SHA-256 of the extracted executable (`depot/release.rs`). The executable is only reused or run when `depot/checksum.rs` confirms it.

### `mods`

- `request` parses `Name:constraint` input.
- `catalog` queries the ficsit.app GraphQL API (`graphql/resolve_mods.graphql`) and filters out versions that do not work with the installed game.
- `resolver` picks the highest version of every mod that satisfies the constraints of everything depending on it, fetching dependencies it has not seen yet.
- `installer` downloads the packages and extracts them into `FactoryGame/Mods`, refusing mod names that could escape that directory.

### `server`

`settings` defines the server options, `arguments` turns them into Unreal command line arguments, and `permissions` restores execute bits that depot downloads do not preserve. On `aarch64` the server is started through `box64`.

## Layout rules

- A module is a file, or a file plus a directory of the same name. `mod.rs` is not used.
- A file that grows past 250 lines, or that mixes unrelated blocks of code, becomes a submodule.
- Library code never panics; failures travel through `Error`.
- Output goes through the `log` crate.

These rules are enforced by clippy (`Cargo.toml`, `clippy.toml`) and pre-commit.

## Container

The `Dockerfile` builds the binary and ships it in a Debian image together with `docker/init.sh`. On `aarch64` the image also installs a checksum-verified box64 package.
