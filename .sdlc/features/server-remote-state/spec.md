# Spec: All state ops available over HTTP for remote consumers

## Problem

Remote agents and tooling that operate against the sdlc HTTP server need to perform all the same lifecycle operations that the local CLI exposes. Currently the server is missing two critical operations:

1. **`artifact draft`** — marking an artifact as written (draft status) so the state machine recognizes it and issues an `approve_*` directive. Without this endpoint, remote agents must shell out to the CLI or use other workarounds to advance a feature from "missing artifact" to "artifact needs approval."

2. **`feature merge`** — finalizing a feature in the `merge` phase by transitioning it to `released` and recording the action in project state. Without this endpoint, remote agents cannot complete the full lifecycle loop over HTTP.

These two gaps mean a remote consumer (an agent operating via the tunnel) cannot drive a feature from `draft` phase all the way to `released` purely through HTTP calls. The server is not a complete remote PM interface without them.

## Solution

Add two HTTP endpoints:

1. `POST /api/artifacts/:slug/:type/draft` — marks the named artifact as draft status (equivalent to `sdlc artifact draft <slug> <type>`).
2. `POST /api/features/:slug/merge` — finalizes the merge phase, transitions feature to `released`, and records the merge action in project state (equivalent to `sdlc merge <slug>`).

Both endpoints follow the existing patterns in `artifacts.rs` and `features.rs` respectively.

## Endpoint Contracts

### POST /api/artifacts/:slug/:type/draft

**Request:** No body required (or empty JSON object `{}`).

**Response (200):**
```json
{
  "slug": "my-feature",
  "artifact_type": "spec",
  "status": "draft",
  "transitioned_to": "specified"   // optional — only present if a phase transition fired
}
```

**Response (404):** When the feature slug does not exist or the artifact type is unknown.

**Response (500):** On I/O or state machine errors.

**Behavior:**
- Loads the feature, calls `feature.mark_artifact_draft(artifact_type)`, saves, runs `try_auto_transition`.
- Returns the optional `transitioned_to` phase string when an auto-transition fires.

### POST /api/features/:slug/merge

**Request:** No body required (or empty JSON object `{}`).

**Response (200):**
```json
{
  "slug": "my-feature",
  "phase": "released",
  "merged": true
}
```

**Response (400):** When the feature is not in the `merge` phase (mirrors CLI behavior: "cannot finalize merge from phase X; move it to 'merge' first").

**Response (404):** When the feature slug does not exist.

**Response (500):** On I/O or state machine errors.

**Behavior:**
- Loads the feature, asserts `phase == merge`, calls `feature.transition(Released, &config)`, saves.
- Loads project state, calls `state.record_action(slug, ActionType::Merge, Phase::Released, "merged")`, calls `state.complete_directive(slug)`, saves state.

## Implementation Notes

- `POST /api/artifacts/:slug/:type/draft` handler lives in `crates/sdlc-server/src/routes/artifacts.rs` alongside the existing approve/reject/waive handlers.
- `POST /api/features/:slug/merge` handler lives in `crates/sdlc-server/src/routes/features.rs` alongside the existing transition handler.
- Both routes are registered in `crates/sdlc-server/src/lib.rs`.
- For the merge endpoint, a 400 response (not 500) is returned when the phase precondition fails — use a new `AppError::bad_request()` helper or encode the 400 status directly. Check how `AppError` currently supports status codes before adding new helpers.
- Follow existing patterns: `spawn_blocking`, `?` propagation, no `unwrap()`.

## Out of Scope

- No changes to existing endpoints.
- No changes to the CLI commands.
- No changes to `sdlc-core` types or state machine rules.
- No frontend changes — these are API-only endpoints consumed by agents.
