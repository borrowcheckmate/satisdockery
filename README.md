# SatisDockery

[![CI](https://github.com/borrowcheckmate/satisdockery/actions/workflows/ci.yml/badge.svg)](https://github.com/borrowcheckmate/satisdockery/actions/workflows/ci.yml)
[![Release](https://img.shields.io/github/v/release/borrowcheckmate/satisdockery?display_name=tag)](https://github.com/borrowcheckmate/satisdockery/releases)
[![Container image](https://img.shields.io/badge/ghcr.io-satisdockery-2496ED?logo=docker&logoColor=white)](https://github.com/borrowcheckmate/satisdockery/pkgs/container/satisdockery)
[![License: EUPL-1.2](https://img.shields.io/badge/license-EUPL--1.2-blue)](LICENSE)

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
| `--experimental` | off | Install the experimental Steam branch |
| `--skip-update` | off | Do not download or update the server |
| `--skip-run` | off | Prepare the installation but do not start the server |
| `--players` | `4` | Maximum number of players |
| `--game-port` | `7777` | Game and HTTPS API port (UDP and TCP) |
| `--messaging-port` | `8888` | Messaging port (TCP) |
| `--max-autosaves` | `5` | Number of rotating autosaves |
| `--max-objects` | `2162688` | Maximum number of objects in the world |
| `--max-tickrate` | `30` | Maximum tick rate |
| `--max-timeout` | `10` | Seconds until an unresponsive player is kicked |
| `--enable-serverstreaming` | off | Only stream part of the world to clients |
| `--disable-seasonalevents` | off | Disable seasonal events |
| `--idle-timeout` | off | Seconds without players before saving and stopping the game |
| `--trust-local-api` | off | Allow tokenless admin API access from loopback |
| `--idle-api-token-file` | none | Application API token file for idle shutdown or the terminal console |
| `--console` | off | Forward terminal input to the server console |
| `--additional-dedicatedserver-args` | none | Extra Unreal arguments, comma separated |

Every option can also be set through an environment variable named `SATISDOCKERY_` followed by the option in upper case, for example `SATISDOCKERY_PLAYERS=8` or `SATISDOCKERY_MODS=RefinedPower:>=4.0.44`. Switches take `true` or `false`. Run `satisdockery --help` for the authoritative list.

By default, the updater uses the public Steam branch. Set `SATISDOCKERY_EXPERIMENTAL=true` to switch to experimental. Unset it or set it to `false` and run the updater again to return to public. Back up your saves before switching branches.

### Managed mods

SatisDockery keeps matching managed versions and removes managed mods and dependencies that are no longer resolved. An empty mod list removes all managed mods. A mod installed by hand or by an older SatisDockery version has no ownership record. It stays in place and can be reused if its descriptor reports the requested version. To let SatisDockery manage it, move that directory out of `FactoryGame/Mods` and restart with the mod listed.

### Logs

SatisDockery logs the update, mod reconciliation, and server phases with timestamped one-line messages. Set `RUST_LOG=debug` for resolution details. DepotDownloader and the game keep their native output.

## Docker

```yaml
services:
  satisfactory:
    image: ghcr.io/borrowcheckmate/satisdockery:latest
    ports:
      - "7777:7777/udp"
      - "7777:7777/tcp"
      - "8888:8888/tcp"
    environment:
      SATISDOCKERY_PLAYERS: "8"
      SATISDOCKERY_MODS: "RefinedPower:>=4.0.44"
    volumes:
      - satisfactory:/home/satisdockery/satisfactory

volumes:
  satisfactory:
```

The server is downloaded into the volume on the first start and updated on every start.

## Local API access

For a container on Docker's default bridge network, set `SATISDOCKERY_TRUST_LOCAL_API=true` to use idle shutdown or the terminal console without creating a token. SatisDockery enables the game's loopback-only admin access when either feature is active. Any process that can connect through the game's loopback interface gets admin API access.

Use token authentication with host networking, shared network namespaces, or a proxy that connects to the game over loopback. In Server Manager's console, run `server.GenerateAPIToken` and mount the resulting token file as a read-only secret. Set `SATISDOCKERY_IDLE_API_TOKEN_FILE` to its path inside the container. A configured token file takes precedence over the loopback setting.

## Idle shutdown and UDP wake

1. Start the server normally, claim it, and set **Auto-Load Session Name** to the session you want to resume.
2. Configure [local API access](#local-api-access) and set `SATISDOCKERY_IDLE_TIMEOUT=900`. Publish the game port over both UDP and TCP as shown above.

SatisDockery checks the game over HTTPS on loopback. The server stays up if a poll or save fails, or if the active session differs from the session set to autoload. Each session has a dedicated save with the prefix `SatisDockery_Idle_`.

After an idle shutdown, the container stays up. A valid UDP status poll starts the game again. Opening Server Manager or running a UDP status monitor can wake it even if nobody joins. If the first join times out, wait for the world to load and try again.

## Attached server console

Configure [local API access](#local-api-access). Set `SATISDOCKERY_CONSOLE=true` and enable `stdin_open: true` and `tty: true` in Compose. With `docker run`, use `-it` instead.

Attach with `docker attach <container>`. Each nonempty line goes to the game's HTTPS console API, and the response appears in the container output. Use Ctrl-P then Ctrl-Q to detach. If the game is asleep, wake it through the game client before sending console commands.

## Migrating from Wolveix's image

Replace the image with `ghcr.io/borrowcheckmate/satisdockery:latest`.

## Documentation

- [ARCHITECTURE.md](ARCHITECTURE.md) explains how the code is organised.
- [CONTRIBUTING.md](CONTRIBUTING.md) explains how to develop, test and release.
- [AGENTS.md](AGENTS.md) holds the rules for automated coding agents.

## License

[EUPL-1.2](LICENSE)
