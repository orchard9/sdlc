# Spec: Wire up Thread Detail Actions

## Problem

The thread detail pane (`ThreadDetailPane.tsx`) has three action stubs that do nothing:

1. **Synthesize** — disabled button, `cursor-not-allowed`, hardcoded tooltip "coming soon"
2. **Promote to Ponder** — disabled button, `cursor-not-allowed`, hardcoded tooltip "coming soon"
3. **Delete** — not present at all

Root cause: no callback props on the component, no `api.deleteThread` / `api.synthesizeThread` / `api.promoteThread` methods in `client.ts`, and no synthesize/promote server routes. The DELETE backend exists (`DELETE /api/threads/:id`, tested) but is unreachable from the UI.

## Goals

Wire all three actions so users can:

1. **Delete** a thread from the detail pane with a confirmation step, then navigate back to the thread list.
2. **Synthesize** a thread — change its status to `synthesized` so the detail pane shows the status badge correctly.
3. **Promote to Ponder** — create a ponder entry from the thread and navigate to it; mark thread status as `promoted`.

## Non-Goals

- Agent-driven synthesis (summarizing comment content with LLM) — not in this fix; status update is enough.
- Bulk-delete from the list view.
- Editing thread title or body in the detail pane.

## Backend Changes

### 1. Add `status` field to `FeedbackThread`

`crates/sdlc-core/src/feedback_thread.rs` — add `status: String` (default `"open"`) to `FeedbackThread`. Persist to `manifest.yaml`. Existing threads without the field deserialize to `"open"` via `serde(default)`.

### 2. `PATCH /api/threads/:id` — update thread fields

New handler `patch_thread` in `crates/sdlc-server/src/routes/threads.rs`:

```
PATCH /api/threads/:id
Body: { "status": "synthesized" | "promoted" | "open" }
Returns: updated thread JSON (same shape as GET /api/threads/:id)
```

Update `save_manifest` to support partial field updates. Add `updated_at` bump on write.

### 3. `POST /api/threads/:id/promote` — promote to ponder

New handler in threads routes:

```
POST /api/threads/:id/promote
Body: {} (empty)
Returns: { "ponder_slug": "<slug>", "thread_id": "<id>" }
```

Logic:
- Load thread
- Create ponder entry via `ponder::create_entry` using thread title + body as brief
- `PATCH` thread status to `"promoted"`, set `promoted_to` = ponder slug in manifest
- Return both slugs

Add `promoted_to: Option<String>` to `FeedbackThread` struct (serde default None).

### 4. Wire DELETE route to return 204-compatible JSON

Existing `DELETE /api/threads/:id` already exists and returns `{"deleted": true}` — no changes needed.

## Frontend Changes

### 5. `api.deleteThread`, `api.patchThread`, `api.promoteThreadToPonder` in `client.ts`

```ts
deleteThread: (slug: string) =>
  request<{ deleted: boolean }>(`/api/threads/${encodeURIComponent(slug)}`, { method: 'DELETE' }),

patchThread: (slug: string, patch: { status?: ThreadStatus }) =>
  request<ThreadDetail>(`/api/threads/${encodeURIComponent(slug)}`, {
    method: 'PATCH', body: JSON.stringify(patch),
  }),

promoteThreadToPonder: (slug: string) =>
  request<{ ponder_slug: string; thread_id: string }>(
    `/api/threads/${encodeURIComponent(slug)}/promote`, { method: 'POST' }
  ),
```

### 6. `ThreadDetailPane` props and button wiring

Add props:

```ts
interface ThreadDetailPaneProps {
  detail: ThreadDetail
  onCommentAdded: (comment: ThreadComment) => void
  onDelete: () => void                      // called after successful delete
  onStatusChange: (updated: ThreadDetail) => void  // called after synthesize
  onPromoted: (ponderSlug: string) => void  // called after promote
}
```

Replace stub buttons:

**Delete button** — small trash icon button, separate from Synthesize/Promote group. On click:
- Show inline confirm state ("Delete? / Cancel") — no modal, just button-label swap
- On confirm: call `api.deleteThread(detail.slug)`, then call `onDelete()`
- Show spinner while deleting; error message on failure

**Synthesize button** — enabled when `detail.status === 'open'`. On click:
- Call `api.patchThread(detail.slug, { status: 'synthesized' })`
- Call `onStatusChange(updatedDetail)`
- Disable button once status is `synthesized`

**Promote to Ponder button** — enabled when `detail.status !== 'promoted'`. On click:
- Call `api.promoteThreadToPonder(detail.slug)`
- Call `onPromoted(ponderSlug)` — parent navigates to `/ponder/<slug>`

### 7. Parent thread page passes callbacks

The parent component rendering `ThreadDetailPane` (likely the threads page or route) must:
- Pass `onDelete` → navigate to `/threads`
- Pass `onStatusChange` → update local state
- Pass `onPromoted` → navigate to `/ponder/<slug>`

## Acceptance Criteria

- [ ] Delete button appears in thread detail header
- [ ] Clicking Delete shows confirm state; confirmed delete removes thread and navigates to `/threads`
- [ ] Synthesize button is enabled for open threads, calls PATCH, updates status badge to "synthesized"
- [ ] Promote to Ponder button calls POST /promote, creates ponder entry, navigates to `/ponder/<slug>`
- [ ] All three actions show loading state during the async call
- [ ] All three actions show an error message on failure (no crash)
- [ ] `SDLC_NO_NPM=1 cargo test --all` passes
- [ ] `cargo clippy --all -- -D warnings` passes
