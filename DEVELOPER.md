# Developer Guide

## Prerequisites

- [Rust](https://rustup.rs) (stable — see `rust-toolchain.toml`)
- [Node.js ≥ 18](https://nodejs.org)
- **PostgreSQL 14+** (optional — cluster mode only): set `DATABASE_URL=postgres://...` to use postgres for telemetry and orchestrator storage. Not needed for local `sdlc ui`.
- [just](https://github.com/casey/just) (task runner)

**Bootstrap from scratch (no tooling installed yet):**

Use `install-deps.ps1` — a single PowerShell script that installs Rust, Node.js, and `just` for you.

```bash
# Linux / macOS (requires pwsh — install once)
sudo apt install powershell   # or: brew install --cask powershell
pwsh install-deps.ps1

# Windows (pwsh is built-in on Win10+)
pwsh install-deps.ps1
```

Then open a new shell and run `just install`.

## Install

**All platforms — using `just`:**

```bash
just install
```

Builds the frontend, installs `ponder` to `~/.cargo/bin`, creates the `sdlc` alias, and installs `orch-tunnel`.

Install `just` manually if you prefer:
```bash
cargo install just   # or: brew install just  |  winget install just
```

**Other recipes:**

```bash
just deps    # print bootstrap instructions (install-deps.ps1 / packaging-deps.ps1)
just build   # build without installing
just test    # cargo test --all (skips npm build)
just lint    # clippy + tsc
just dist    # build platform packages (.tar.gz, .deb, .rpm on Linux) — requires pwsh packaging-deps.ps1 first
just clean   # remove build artifacts
just         # list all recipes
```

> `just test` sets `SDLC_NO_NPM=1` automatically — without it, `cargo test` hangs if `frontend/dist` is absent.

## First Steps

```bash
cd your-project
sdlc init
sdlc ui
```

`sdlc init` creates `.sdlc/`, injects `AGENTS.md`, and installs slash commands for Claude Code, Gemini CLI, and OpenCode. `sdlc ui` opens the dashboard — all state is live via SSE, no refresh needed.

From here, create a feature and see the raw directive the machine emits:

```bash
sdlc feature create auth-login --title "OAuth login"
sdlc next --for auth-login --json
```

## Dev Loop (hot reload)

Two terminals. Run from the sdlc repo root.

The target project must be initialized first:
```bash
cd /path/to/your-project && sdlc init
```

**Terminal 1** — Rust backend, recompiles on save:

macOS / Linux:
```bash
SDLC_ROOT=/path/to/your-project \
cargo watch -x 'run --bin ponder -- ui --port 3141 --no-open'
```

Windows (PowerShell):
```powershell
$env:SDLC_ROOT = "C:\path\to\your-project"
cargo watch -x 'run --bin ponder -- ui --port 3141 --no-open'
```

**Terminal 2** — Vite dev server with React HMR:
```bash
cd frontend && npm run dev
```

Open `http://localhost:5173`. Vite proxies `/api` to the Rust backend on port 3141. React changes reflect instantly; Rust changes trigger a recompile.

`SDLC_ROOT` points the backend at any project on disk — no need to `cd` into it.

> **Stale server?** If a previous `sdlc ui` process is already bound to port 3141 (check with `sdlc list`), kill it first — it may be pointing at the wrong project or running an old binary.

## Hub Dev Loop (Fleet UI)

Test hub mode locally with Postgres and Gitea in Docker. sdlc-server runs natively for hot reload.

**One-time setup:**

```bash
docker compose up -d
./dev/setup-gitea.sh        # creates admin user, orchard9 org, prints API token
mkdir -p /tmp/sdlc-hub/.sdlc
```

**Three terminals:**

```bash
# Terminal 1 — Postgres + Gitea
docker compose up -d

# Terminal 2 — Rust backend in hub mode
SDLC_ROOT=/tmp/sdlc-hub \
SDLC_HUB=true \
GITEA_URL=http://localhost:7782 \
GITEA_API_TOKEN=<from setup-gitea.sh> \
DATABASE_URL=postgres://sdlc:sdlc@localhost:7781/sdlc \
cargo watch -x 'run --bin ponder -- ui start --port 7778 --no-open --no-tunnel --hub'

# Terminal 3 — Vite dev server
cd frontend && npm run dev
```

Open `http://localhost:5173`. Auth is bypassed on localhost. Use **Import** (paste a clone URL) or **Create** (enter a repo name) in the hub UI to add repos — they're mirrored into the local Gitea automatically.

To test a running project instance alongside the hub, start a second server pointed at the project:

```bash
SDLC_ROOT=/path/to/your-project \
SDLC_HUB_URL=http://localhost:7778 \
cargo run --bin ponder -- ui start --port 7777 --no-open --no-tunnel
```

The project instance sends heartbeats to the hub and appears in the fleet view.

### Tier 2: Woodpecker CI (provision flow)

The docker-compose stack includes Woodpecker server + agent for testing the full provision flow (hub UI "Start" button → pipeline → pod deploy). Woodpecker requires a Gitea OAuth app and its own setup:

```bash
# After docker compose up -d and setup-gitea.sh:
./dev/setup-woodpecker.sh    # OAuth flow, API token, repo activation
```

Add the printed `WOODPECKER_URL` and `WOODPECKER_API_TOKEN` to your hub server env. Note: Woodpecker tokens are invalidated on server restart — re-run the script after `docker compose restart woodpecker-server`.

The agent is configured with `WOODPECKER_BACKEND_DOCKER_NETWORK=sdlc_default` so pipeline containers can reach Gitea via Docker service names.

**Tear down:**

```bash
docker compose down -v   # -v removes volumes (pgdata, gitea-data, woodpecker-data)
```

## Going Deeper

[`docs/architecture.md`](docs/architecture.md) — codebase layout, data schemas, classifier, REST API, contributing (rules, commands, action types).
