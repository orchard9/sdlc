# Tasks: Wire up Thread Detail Actions

## T1 — Add `status` and `promoted_to` fields to `FeedbackThread`
- File: `crates/sdlc-core/src/feedback_thread.rs`
- Add `status: String` with `#[serde(default = "default_status")]` → `"open"`
- Add `promoted_to: Option<String>` with `#[serde(default, skip_serializing_if = "Option::is_none")]`
- Update `create_thread` to set `status = "open"`, `promoted_to = None`
- Update `thread_to_json` helper in `threads.rs` to include both fields in JSON

## T2 — Add `PATCH /api/threads/:id` route (status update)
- File: `crates/sdlc-server/src/routes/threads.rs`
- Add `PatchBody { status: Option<String> }` struct
- Add `patch_thread` handler: load thread, update status + `updated_at`, save, return updated JSON
- Register route in `crates/sdlc-server/src/routes/mod.rs`

## T3 — Add `POST /api/threads/:id/promote` route
- File: `crates/sdlc-server/src/routes/threads.rs`
- Load thread, call `sdlc_core::ponder::create_entry(root, slug, title, brief)` using thread title + body
- Update thread: `status = "promoted"`, `promoted_to = ponder_slug`, save
- Return `{ "ponder_slug": "<slug>", "thread_id": "<id>" }`
- Register route in `mod.rs`

## T4 — Add API client methods
- File: `frontend/src/api/client.ts`
- Add `deleteThread(slug)` → `DELETE /api/threads/:slug`
- Add `patchThread(slug, patch)` → `PATCH /api/threads/:slug`
- Add `promoteThreadToPonder(slug)` → `POST /api/threads/:slug/promote`

## T5 — Wire up ThreadDetailPane buttons
- File: `frontend/src/components/threads/ThreadDetailPane.tsx`
- Add `onDelete`, `onStatusChange`, `onPromoted` props
- Replace "Synthesize" stub: enabled when `status === 'open'`, calls `patchThread`, then `onStatusChange`
- Replace "Promote to Ponder" stub: enabled when `status !== 'promoted'`, calls `promoteThreadToPonder`, then `onPromoted`
- Add Delete button (trash icon): shows inline confirm state, calls `deleteThread`, then `onDelete`
- Add loading states and inline error messages for all three actions

## T6 — Pass callbacks from parent thread page
- Find the page/component rendering `ThreadDetailPane` (threads route)
- Pass `onDelete` → navigate to `/threads`
- Pass `onStatusChange` → update local thread state
- Pass `onPromoted` → navigate to `/ponder/<slug>`

## T7 — Verify build passes
- Run `SDLC_NO_NPM=1 cargo test --all`
- Run `cargo clippy --all -- -D warnings`
- Run `cd frontend && npm run build` (or type-check)
