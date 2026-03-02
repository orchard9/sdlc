# Design: Blocked Feature UX — BlockedPanel

## Overview

This document describes the component structure, API contract, and interaction design for
the `BlockedPanel` component and the `remove_blocker` backend method.

The design is minimal by intent (~125 lines of new code total) and is consistent with all
existing patterns in the codebase.

---

## Backend Design

### `sdlc-core/src/feature.rs` — `remove_blocker`

```rust
pub fn remove_blocker(&mut self, idx: usize) -> Result<()> {
    if idx >= self.blockers.len() {
        return Err(SdlcError::InvalidInput(format!(
            "blocker index {} out of range (len={})",
            idx,
            self.blockers.len()
        )));
    }
    self.blockers.remove(idx);
    self.updated_at = Utc::now();
    Ok(())
}
```

### `sdlc-server/src/routes/features.rs` — DELETE handler

```rust
#[derive(serde::Deserialize, Default)]
pub struct RemoveBlockerBody {
    pub reason: Option<String>,
}

pub async fn remove_blocker(
    State(app): State<AppState>,
    Path((slug, idx)): Path<(String, usize)>,
    Json(body): Json<RemoveBlockerBody>,
) -> Result<Json<serde_json::Value>, AppError> {
    let root = app.root.clone();
    let result = tokio::task::spawn_blocking(move || {
        let mut feature = sdlc_core::feature::Feature::load(&root, &slug)?;
        feature.remove_blocker(idx)?;
        // Optional: store reason as a decision comment
        if let Some(reason) = body.reason.filter(|r| !r.trim().is_empty()) {
            let comment = sdlc_core::comment::Comment::new(
                feature.next_comment_seq,
                format!("Blocker removed: {reason}"),
                Some("decision".to_string()),
                None,
            );
            feature.comments.push(comment);
            feature.next_comment_seq += 1;
        }
        feature.save(&root)?;
        Ok::<_, sdlc_core::SdlcError>(serde_json::json!({ "ok": true }))
    })
    .await
    .map_err(|e| AppError(anyhow::anyhow!("task join error: {e}")))??;
    Ok(Json(result))
}
```

### Route registration in `sdlc-server/src/lib.rs`

```rust
.route(
    "/api/features/{slug}/blockers/{idx}",
    delete(routes::features::remove_blocker),
)
```

SSE auto-refresh: when `feature.save()` is called, the mtime watcher detects the file
change and emits `SseMessage::Update`. The frontend re-fetches feature data on `Update`
via the existing `useFeature` hook. No new SSE variant is needed.

---

## Frontend Design

### Component: `BlockedPanel.tsx`

**Location:** `frontend/src/components/features/BlockedPanel.tsx`

**Props:**
```ts
interface BlockedPanelProps {
  slug: string
  blockers: string[]
  allSlugs: string[]    // used to detect in-project slug links
  isRunning: boolean
  onRunWithDirection: (direction: string) => void
}
```

**State:**
- `removingIdx: number | null` — which blocker has its inline reason UI open
- `reasons: Record<number, string>` — per-blocker reason inputs
- `direction: string` — the direction input value
- `submitting: number | null` — index of blocker being deleted (for loading state)

**Interaction flow:**

1. User sees blocker list. Each row:
   - Text of blocker
   - If text matches a slug in `allSlugs`: a `→ slug` link to `/features/slug`
   - `[Remove]` button

2. Clicking `[Remove]` on row `i`:
   - Sets `removingIdx = i`
   - Reveals: `[reason input] [Confirm] [Cancel]`

3. Clicking `[Confirm]`:
   - Sets `submitting = i`
   - Calls `DELETE /api/features/:slug/blockers/:i` with optional `{ reason }`
   - On success: SSE `Update` → `useFeature` refetches → panel re-renders with blocker gone
   - Resets `removingIdx`, `submitting`

4. Direction input + Run:
   - Controlled text input bound to `direction`
   - `[Run with direction]` disabled when `direction.trim() === ''` or `isRunning`
   - On click: calls `onRunWithDirection(direction)` → parent calls `startRun` with
     `context: direction` appended to the request body

**Wireframe:**

```
┌─ ⚠ Blocked ────────────────────────────────────────────────────┐
│                                                                  │
│  • "waiting for product-content"  [→ product-content]           │
│  • "auth-setup must complete"                                    │
│    [✕ Remove] [reason: _________________ ] [Confirm] [Cancel]   │
│                                                                  │
│  ──────────────────────────────────────────────────────────── │
│  Direction for the agent                                         │
│  [skip auth-setup, use env vars                              ]   │
│                                                                  │
│  [▶ Run with direction]                                          │
└──────────────────────────────────────────────────────────────────┘
```

Visual treatment:
- Panel background: `bg-amber-500/10 border border-amber-500/30 rounded-xl`
- Header: amber `AlertTriangle` icon + "Blocked" label
- Direction section: subtle divider, muted label
- Run button: primary styling, disabled state when empty or running

### Integration in `FeatureDetail.tsx`

**Add after the header block, before the "Next action" card:**

```tsx
{feature.blocked && (
  <BlockedPanel
    slug={slug}
    blockers={feature.blockers}
    allSlugs={allFeatureSlugs}
    isRunning={running}
    onRunWithDirection={(direction) => {
      startRun({
        key: slug,
        runType: 'feature',
        target: slug,
        label: slug,
        startUrl: `/api/run/${slug}`,
        stopUrl: `/api/run/${slug}/stop`,
        context: direction,
      })
    }}
  />
)}
```

`allFeatureSlugs` is derived from the existing `useFeatures` hook (or passed in from a
state fetch). The `startRun` function in `AgentRunContext` already accepts a `context`
field that is included in the POST body to `/api/run/:slug`.

---

## Data flow diagram

```
User clicks "Run with direction"
        │
        ▼
FeatureDetail calls startRun({ context: direction })
        │
        ▼
AgentRunContext POSTs to /api/run/:slug with { context }
        │
        ▼
Server spawns agent run with direction in system prompt
        │
        ▼
SSE stream opens → AgentPanel shows activity
        │
        ▼
Agent calls sdlc next, sees unblock_dependency, uses direction to act

User clicks "Remove" on a blocker
        │
        ▼
BlockedPanel calls DELETE /api/features/:slug/blockers/:idx
        │
        ▼
Server: remove_blocker() → feature.save()
        │
        ▼
mtime watcher detects change → SseMessage::Update
        │
        ▼
Frontend re-fetches feature → blockers list updated
```

---

## File change summary

| File | Change |
|---|---|
| `crates/sdlc-core/src/feature.rs` | Add `remove_blocker` method + test |
| `crates/sdlc-server/src/routes/features.rs` | Add `remove_blocker` handler + `RemoveBlockerBody` |
| `crates/sdlc-server/src/lib.rs` | Register DELETE route |
| `frontend/src/components/features/BlockedPanel.tsx` | New component (~80 lines) |
| `frontend/src/pages/FeatureDetail.tsx` | Conditionally render BlockedPanel |

Total estimated lines: ~125.

No new types in `types.ts` needed — `blockers: string[]` is already in `FeatureDetail`.
No new SSE variants. No CLI changes in v1.
