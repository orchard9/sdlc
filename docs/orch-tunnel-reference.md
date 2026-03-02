# orch-tunnel Reference

`orch-tunnel` is the tunnel binary used by `sdlc ui --tunnel` to expose the SDLC server publicly.

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

`sdlc ui --tunnel` spawns `orch-tunnel` with the project name from `.sdlc/config.yaml`:

```bash
orch-tunnel http 3141 --name sdlc
# → https://sdlc.tunnel.threesix.ai
```

The project name is passed as `--name`, giving a predictable URL that matches the project. The tunnel URL is extracted from `orch-tunnel`'s stderr output and validated by checking for `.tunnel.threesix.ai`.

## Install

```bash
brew install orch-tunnel
```
