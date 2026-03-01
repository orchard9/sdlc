# Developer Guide

## Prerequisites

- [Rust](https://rustup.rs) (stable — see `rust-toolchain.toml`)
- [Node.js ≥ 18](https://nodejs.org)

## Install

```bash
make install
```

Builds the frontend, installs the `sdlc` binary to your Cargo bin, and installs `cloudflared` (needed for `sdlc ui --tunnel`).

Other targets:

```bash
make build   # build without installing
make test    # SDLC_NO_NPM=1 cargo test --all
make lint    # clippy + tsc
make clean   # remove build artifacts
```

> `make test` skips the npm build — without `SDLC_NO_NPM=1`, `cargo test` hangs if `frontend/dist` is absent.

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
```bash
SDLC_ROOT=/path/to/your-project \
cargo watch -x 'run --bin sdlc -- ui --port 3141 --no-open'
```

**Terminal 2** — Vite dev server with React HMR:
```bash
cd frontend && npm run dev
```

Open `http://localhost:5173`. Vite proxies `/api` to the Rust backend on port 3141. React changes reflect instantly; Rust changes trigger a recompile.

`SDLC_ROOT` points the backend at any project on disk — no need to `cd` into it.

## Going Deeper

[`docs/architecture.md`](docs/architecture.md) — codebase layout, data schemas, classifier, REST API, contributing (rules, commands, action types).
