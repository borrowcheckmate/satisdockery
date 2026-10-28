# SatisDockery

A Satisfactory dedicated server in Docker, with a Rust wrapper that downloads the server through DepotDownloader, resolves and installs mods from [ficsit.app](https://ficsit.app), and then starts the server.

It runs on `x86_64` and on `aarch64` (through box64).

## Compared to Wolveix's Satisfactory server

[Wolveix's image](https://github.com/wolveix/satisfactory-server) works great. This one adds `aarch64` support and automatic mod installation. Wolveix's work made it clear how the server setup works.

## Usage

```sh
satisdockery --path /home/satisdockery/satisfactory --mods "RefinedPower:>=4.0.44,SecondMod:>=1.0.10"
```

| Option | Default | Description |
| --- | --- | --- |
| `--path` | required | Satisfactory installation directory |
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

Replace the image with `ghcr.io/borrowcheckmate/satisdockery:latest`.

## Documentation

- [ARCHITECTURE.md](ARCHITECTURE.md) explains how the code is organised.
- [CONTRIBUTING.md](CONTRIBUTING.md) explains how to develop, test and release.
- [AGENTS.md](AGENTS.md) holds the rules for automated coding agents.

## License

[EUPL-1.2](LICENSE)
