# orch-tunnel Reference

`orch-tunnel` is the tunnel binary used by `sdlc ui` to expose the SDLC server publicly. The tunnel starts automatically by default — use `sdlc ui --no-tunnel` to run without it.

## Default Behavior

`sdlc ui` starts a public tunnel automatically on launch. The tunnel URL is stable across restarts — the same project always gets the same URL.

```bash
sdlc ui              # tunnel starts automatically (default)
sdlc ui --no-tunnel  # local-only, no tunnel
```

If `orch-tunnel` is not installed or fails to connect, `sdlc ui` logs a warning and continues in local-only mode. It does not fail hard.

## CLI

```
Usage: orch-tunnel <http|https> <port> [flags]
```

## Examples

```bash
orch-tunnel http 3000
orch-tunnel http 3000 --name my-project
orch-tunnel https 3000
```

## Flags

| Flag | Default | Description |
|------|---------|-------------|
| `-name string` | (random slug) | Named tunnel slug — determines the subdomain |
| `-edge string` | `connect.tunnel.threesix.ai:7844` | Edge server address |
| `-version` | — | Print version and exit |

## URL Format

| Mode | URL |
|------|-----|
| Named | `https://{name}.tunnel.threesix.ai` |
| Random | `https://{random-slug}.tunnel.threesix.ai` |

## Usage in sdlc

`sdlc ui` spawns `orch-tunnel` with the project name from `.sdlc/config.yaml`:

```bash
orch-tunnel http 3141 --name sdlc
# → https://sdlc.tunnel.threesix.ai
```

The project name is passed as `--name`, giving a stable URL that matches the project and does not change between restarts. The tunnel URL is extracted from `orch-tunnel`'s stdout output and validated by checking for `.tunnel.threesix.ai`.

## Install

```bash
brew install orch-tunnel
```
