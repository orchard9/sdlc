# Developer Guide

## Prerequisites

- [Rust](https://rustup.rs) (stable — see `rust-toolchain.toml`)
- [Node.js ≥ 18](https://nodejs.org)

## Install

**All platforms — using `just`:**

```bash
just install
```

Builds the frontend, installs `ponder` to `~/.cargo/bin`, creates the `sdlc` alias, and installs `orch-tunnel`.

Install `just` first if you don't have it:
```bash
cargo install just   # or: brew install just  |  winget install just
```

**Other recipes:**

```bash
just build   # build without installing
just test    # cargo test --all (skips npm build)
just lint    # clippy + tsc
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

## Going Deeper

[`docs/architecture.md`](docs/architecture.md) — codebase layout, data schemas, classifier, REST API, contributing (rules, commands, action types).
