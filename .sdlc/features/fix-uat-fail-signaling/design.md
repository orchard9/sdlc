# Design: UAT Fail Endpoint and Skill Template for Explicit Failure Signaling

## Overview

This feature adds a thin server-side signal endpoint and a corresponding SSE variant so
the UAT agent can explicitly mark a UAT run as failed. It also updates the skill template
to instruct the agent to call the endpoint on a `Failed` verdict.

No state transitions occur on failure — the milestone stays in `Verifying`. The endpoint
is purely a notification mechanism.

## Components

### 1. New SSE variant: `MilestoneUatFailed`

**File:** `crates/sdlc-server/src/state.rs`

```rust
/// A milestone UAT agent run completed with a failing verdict.
/// No milestone state change — informational signal only.
MilestoneUatFailed { slug: String },
```

Placed after `MilestoneUatCompleted` in the enum for logical adjacency.

### 2. SSE serialization

**File:** `crates/sdlc-server/src/routes/events.rs`

```rust
Ok(SseMessage::MilestoneUatFailed { slug }) => {
    let data = serde_json::json!({
        "type": "milestone_uat_failed",
        "slug": slug,
    })
    .to_string();
    Some(Ok(Event::default().event("milestone_uat").data(data)))
}
```

Emitted on the same `milestone_uat` channel as `MilestoneUatCompleted` so the
frontend can handle both event types in one place.

### 3. New endpoint: `POST /api/milestone/:slug/uat/fail`

**File:** `crates/sdlc-server/src/routes/runs.rs`

Handler logic:
1. Validate slug (reuse `validate_slug`).
2. Spawn blocking task: load the milestone to verify it exists.
3. Broadcast `SseMessage::MilestoneUatFailed { slug }`.
4. Return `{ slug, status: "failed" }`.

No file writes, no state transitions. The endpoint is idempotent — calling it
multiple times has no side effects beyond emitting another SSE event.

```rust
pub async fn fail_milestone_uat(
    Path(slug): Path<String>,
    State(app): State<AppState>,
) -> Result<Json<serde_json::Value>, AppError> {
    validate_slug(&slug)?;
    // Verify the milestone exists (early error if typo'd slug).
    let root = app.root.clone();
    let slug_clone = slug.clone();
    tokio::task::spawn_blocking(move || sdlc_core::milestone::Milestone::load(&root, &slug_clone))
        .await
        .map_err(|e| AppError(anyhow::anyhow!("task join error: {e}")))??;
    let _ = app.event_tx.send(SseMessage::MilestoneUatFailed { slug: slug.clone() });
    Ok(Json(serde_json::json!({ "slug": slug, "status": "failed" })))
}
```

### 4. Route registration

**File:** `crates/sdlc-server/src/lib.rs`

Added alongside the existing UAT start/stop/events routes:

```rust
.route(
    "/api/milestone/{slug}/uat/fail",
    post(routes::runs::fail_milestone_uat),
)
```

### 5. Frontend type update

**File:** `frontend/src/lib/types.ts`

Extend the union type:

```typescript
export interface MilestoneUatSseEvent {
  type: 'milestone_uat_completed' | 'milestone_uat_failed'
  slug: string
}
```

### 6. UAT skill template update

**File:** `crates/sdlc-cli/src/cmd/init/commands/sdlc_milestone_uat.rs`

Three templates need updating: `SDLC_MILESTONE_UAT_COMMAND` (Claude),
`SDLC_MILESTONE_UAT_PLAYBOOK` (Gemini/OpenCode), and `SDLC_MILESTONE_UAT_SKILL` (Agents).

**Current "On Failed" instruction:**
> do NOT call `milestone complete`. The milestone stays in `Verifying`. Fix the feature tasks, then re-run this command.

**Updated "On Failed" instruction:**
> do NOT call `milestone complete`. Call the fail endpoint to signal the outcome:
> ```bash
> curl -s -X POST http://localhost:7777/api/milestone/<slug>/uat/fail
> ```
> The milestone stays in `Verifying`. Fix the feature tasks, then re-run.

The same curl instruction is added to Step 5 of the command template, the Gemini playbook
step 6, and the Agents SKILL.md workflow.

## Data Flow

```
UAT Agent (Failed verdict)
  │
  ├── writes summary.md
  ├── writes uat_results.md
  └── POST /api/milestone/<slug>/uat/fail
        │
        ├── validates slug
        ├── loads milestone (existence check)
        ├── broadcasts MilestoneUatFailed { slug }
        │       │
        │       └── SSE: event=milestone_uat data={type:"milestone_uat_failed",slug}
        │
        └── returns { slug, status: "failed" }
```

## No-Change Items

- Milestone `Verifying` status is unchanged (correct — outstanding failures remain).
- No `run.yaml` record changes — the UAT agent already writes that with verdict=`failed`.
- No frontend UI changes (consumers can now detect the event; how to display is out of scope).
- No CLI changes beyond the skill template text.

## Testing

Unit tests in `runs.rs`:
- `fail_milestone_uat_returns_slug_and_status` — HTTP layer test verifying 200 + body.

All existing tests continue to pass; no behavioral changes to the pass path.
