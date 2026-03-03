# Design: sdlc ui --run-actions (Invert --no-orchestrate Default)

## Overview

This is a targeted rename-and-inversion of one flag in two files. There is no new behavior, no architectural change, and no data model change. The design is purely about CLI interface semantics.

## Change Map

### File 1: `crates/sdlc-cli/src/main.rs`

**`Commands::Ui` variant** — remove `no_orchestrate`, add `run_actions`:

```rust
// Before
Ui {
    #[arg(long, default_value = "0")]
    port: u16,
    #[arg(long)]
    no_open: bool,
    #[arg(long)]
    tunnel: bool,
    #[arg(long, default_value_t = 60)]
    tick_rate: u64,
    /// Skip starting the orchestrator daemon
    #[arg(long)]
    no_orchestrate: bool,
    #[command(subcommand)]
    subcommand: Option<UiSubcommand>,
},

// After
Ui {
    #[arg(long, default_value = "0")]
    port: u16,
    #[arg(long)]
    no_open: bool,
    #[arg(long)]
    tunnel: bool,
    #[arg(long, default_value_t = 60)]
    tick_rate: u64,
    /// Start the orchestrator daemon and execute scheduled actions
    #[arg(long)]
    run_actions: bool,
    #[command(subcommand)]
    subcommand: Option<UiSubcommand>,
},
```

**Dispatch arm** — thread `run_actions` through to `cmd::ui::run`:

```rust
// Before
Commands::Ui {
    port, no_open, tunnel, tick_rate, no_orchestrate, subcommand,
} => cmd::ui::run(&root, subcommand, port, no_open, tunnel, tick_rate, no_orchestrate),

// After
Commands::Ui {
    port, no_open, tunnel, tick_rate, run_actions, subcommand,
} => cmd::ui::run(&root, subcommand, port, no_open, tunnel, tick_rate, run_actions),
```

---

### File 2: `crates/sdlc-cli/src/cmd/ui.rs`

**`UiSubcommand::Start` variant** — same rename:

```rust
// Before
Start {
    #[arg(long, default_value = "3141")]
    port: u16,
    #[arg(long)]
    no_open: bool,
    #[arg(long)]
    tunnel: bool,
    #[arg(long, default_value_t = 60)]
    tick_rate: u64,
    /// Skip starting the orchestrator daemon
    #[arg(long)]
    no_orchestrate: bool,
},

// After
Start {
    #[arg(long, default_value = "3141")]
    port: u16,
    #[arg(long)]
    no_open: bool,
    #[arg(long)]
    tunnel: bool,
    #[arg(long, default_value_t = 60)]
    tick_rate: u64,
    /// Start the orchestrator daemon and execute scheduled actions
    #[arg(long)]
    run_actions: bool,
},
```

**`run()` function signature** — rename parameter:

```rust
// Before
pub fn run(root, subcommand, port, no_open, tunnel, tick_rate, no_orchestrate) -> Result<()>

// After
pub fn run(root, subcommand, port, no_open, tunnel, tick_rate, run_actions) -> Result<()>
```

**Dispatch match arm inside `run()`** — update parameter name:

```rust
// Before
Some(UiSubcommand::Start { port: p, no_open: n, tunnel: t, tick_rate: tr, no_orchestrate: no_orch })
    => run_start(root, p, n, t, tr, no_orch),

// After
Some(UiSubcommand::Start { port: p, no_open: n, tunnel: t, tick_rate: tr, run_actions: ra })
    => run_start(root, p, n, t, tr, ra),
```

**`run_start()` function signature** — rename parameter:

```rust
// Before
fn run_start(root, port, no_open, use_tunnel, tick_rate, no_orchestrate) -> Result<()>

// After
fn run_start(root, port, no_open, use_tunnel, tick_rate, run_actions) -> Result<()>
```

**Spawn condition** — invert the guard:

```rust
// Before
if !no_orchestrate {
    // spawn orchestrator thread
}

// After
if run_actions {
    // spawn orchestrator thread
}
```

---

### File 3: `DEVELOPER.md`

Search for `--no-orchestrate` and replace with `--run-actions`. Update any surrounding prose explaining the flag's purpose (from "skip orchestrator" to "enable orchestrator/actions").

---

## No Changes Needed

- Frontend: `run_actions` is a server-process concern; the UI never sees this flag.
- `sdlc-server`: The server does not accept or inspect this flag.
- `sdlc-core`: Pure data layer; unaffected.
- Tests: There are no existing tests that reference `no_orchestrate` in the UI command path.

## Verification Plan

After implementation:
1. `cargo build --all` — compiles cleanly.
2. `SDLC_NO_NPM=1 cargo test --all` — all tests pass.
3. `cargo clippy --all -- -D warnings` — no new warnings.
4. Manual smoke test: `sdlc ui --help` shows `--run-actions`, does not show `--no-orchestrate`.
