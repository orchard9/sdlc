# Spec: UAT Fail Endpoint and Skill Template for Explicit Failure Signaling

## Problem

When a milestone UAT run completes with a **Failed** verdict, the agent has no way to
explicitly signal that failure to the server. The only existing signal path is
`sdlc milestone complete <slug>` (for pass/pass-with-tasks), which emits a
`MilestoneUatCompleted` SSE event. On failure, the skill template currently says
"do NOT call `milestone complete`" and leaves the milestone in `Verifying` — silently.

This means:
- The frontend receives no `milestone_uat` event on failure and cannot react
  (e.g. refresh runs list, show a failure badge, or notify the user).
- Observers polling SSE have no way to distinguish "UAT still running" from
  "UAT finished with a failing verdict".
- The skill template gives the agent no affirmative action on failure, making the
  failure path a no-op from the server's perspective.

## Proposed Solution

### 1. New server endpoint: `POST /api/milestone/:slug/uat/fail`

A lightweight endpoint that:
- Validates the slug.
- Loads the milestone to confirm it exists.
- Emits a `MilestoneUatFailed { slug }` SSE message.
- Returns `{ slug, status: "failed" }`.

The endpoint does **not** change milestone state (the milestone stays in `Verifying`
because there are outstanding failures). It is purely a signal.

### 2. New SSE variant: `MilestoneUatFailed { slug }`

Add to `SseMessage` enum in `state.rs`:

```rust
/// A milestone UAT agent run completed with a failing verdict — no state change,
/// but the frontend can react (refresh runs list, show failure badge).
MilestoneUatFailed { slug: String },
```

Serialize in `events.rs` as:

```
event: milestone_uat
data: { "type": "milestone_uat_failed", "slug": "..." }
```

Using the same `milestone_uat` event channel keeps the frontend handler consolidated.

### 3. Update frontend type

Add `'milestone_uat_failed'` to `MilestoneUatSseEvent.type` union in `frontend/src/lib/types.ts`.

### 4. Update UAT skill template

In `crates/sdlc-cli/src/cmd/init/commands/sdlc_milestone_uat.rs`, update the
"On Failed" branch of the skill template to instruct the agent to call:

```bash
curl -s -X POST http://localhost:7777/api/milestone/<slug>/uat/fail
```

after writing `summary.md` and `uat_results.md`.

## Scope

- `crates/sdlc-server/src/state.rs` — add `MilestoneUatFailed` variant
- `crates/sdlc-server/src/routes/events.rs` — serialize the new variant
- `crates/sdlc-server/src/routes/runs.rs` — add `fail_milestone_uat` handler
- `crates/sdlc-server/src/lib.rs` — register the new route
- `frontend/src/lib/types.ts` — extend `MilestoneUatSseEvent.type`
- `crates/sdlc-cli/src/cmd/init/commands/sdlc_milestone_uat.rs` — update skill templates

## Out of Scope

- No milestone state transitions (milestone stays `Verifying` on failure — that is correct).
- No database changes.
- No frontend UI changes beyond the type update (consumers can decide how to react).

## Acceptance Criteria

1. `POST /api/milestone/:slug/uat/fail` returns 200 with `{ slug, status: "failed" }`.
2. Calling the endpoint emits an SSE event on the `milestone_uat` channel with
   `{ type: "milestone_uat_failed", slug }`.
3. The `MilestoneUatSseEvent` TypeScript type accepts `'milestone_uat_failed'`.
4. The UAT skill template (Claude, Gemini playbook, and Agents SKILL.md variants)
   instructs the agent to call the endpoint when verdict is `Failed`.
5. `SDLC_NO_NPM=1 cargo test --all` passes.
6. `cargo clippy --all -- -D warnings` passes.
