# Spec: sdlc ui --run-actions (Invert --no-orchestrate Default)

## Feature Slug
`dev-driver-run-actions-flag`

## Problem Statement

Currently `sdlc ui` starts the orchestrator daemon by default, and users must pass `--no-orchestrate` to disable it. This is the wrong default for a system that executes autonomous AI-driven actions: running actions should require explicit opt-in, not opt-out.

The orchestrator daemon drives scheduled actions (including the `dev-driver` tool). If it starts unconditionally every time `sdlc ui` is run, actions execute silently without the user's knowledge. This is a safety and trust issue, particularly for teams adopting the system for the first time.

The inversion is simple: by default, `sdlc ui` starts the web server only (no orchestrator). When the user explicitly passes `--run-actions`, the orchestrator starts.

## Goal

Make action execution in `sdlc ui` opt-in by:

1. Removing `--no-orchestrate` from the `Ui` variant in `Commands` in `main.rs` and from `UiSubcommand::Start` in `ui.rs`.
2. Adding `--run-actions: bool` (default `false`) to both `Ui` and `UiSubcommand::Start`.
3. Flipping the spawn logic: start the orchestrator only when `run_actions == true`.
4. Updating `DEVELOPER.md` to reflect the new flag name and semantics.

## Scope

### In Scope

- `crates/sdlc-cli/src/main.rs`: replace `no_orchestrate: bool` with `run_actions: bool` in `Commands::Ui`.
- `crates/sdlc-cli/src/cmd/ui.rs`: replace `no_orchestrate: bool` with `run_actions: bool` in `UiSubcommand::Start`, `run()`, and `run_start()`. Flip the spawn condition from `if !no_orchestrate` to `if run_actions`.
- `DEVELOPER.md`: update any mention of `--no-orchestrate` to `--run-actions`.
- Task T5 (user-gap): evaluate and document any config-file escape hatch for project-wide opt-out (can be a backlog entry if not trivial).

### Out of Scope

- Changing the orchestrator daemon itself.
- UI changes (the frontend does not know about this flag).
- Changing `sdlc orchestrate` subcommand behavior.

## Acceptance Criteria

1. `sdlc ui` (no flags) starts the web server; the orchestrator thread is NOT started; scheduled actions remain in `pending` state.
2. `sdlc ui --run-actions` starts both the web server and the orchestrator thread; scheduled actions execute on schedule.
3. `sdlc ui --no-orchestrate` is no longer a valid flag (removed from CLI); passing it produces a clap error.
4. `sdlc ui start --run-actions` and `sdlc ui start` work equivalently to the top-level form for their respective modes.
5. `DEVELOPER.md` accurately reflects the new flag.
6. Existing `--tunnel`, `--no-open`, `--port`, `--tick-rate` flags are unaffected.

## User-Facing Behavior Change

| Scenario | Before | After |
|---|---|---|
| `sdlc ui` | Orchestrator starts, actions run | Orchestrator does NOT start |
| `sdlc ui --no-orchestrate` | Orchestrator skipped | Flag removed (error) |
| `sdlc ui --run-actions` | Flag not present (error) | Orchestrator starts, actions run |

## Notes

- This is a targeted rename + inversion. No new features, no new capabilities.
- The acceptance test for v21-dev-driver milestone (Test 8) explicitly validates this behavior.
- The `UiSubcommand::Start` variant must also be updated to keep `sdlc ui start --run-actions` consistent with `sdlc ui --run-actions`.
