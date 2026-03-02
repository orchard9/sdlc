# Design: FeedbackThread core — data model, CLI, and REST API

## Architecture Overview

`FeedbackThread` follows the same layered pattern used by `AmaThread` and `KnowledgeEntry`:

- **sdlc-core/src/feedback_thread.rs** — pure data layer: structs, CRUD fns, no HTTP
- **sdlc-cli/src/cmd/thread.rs** — thin CLI wrapper using `clap` subcommands
- **sdlc-server/src/routes/threads.rs** — thin Axum route handlers delegating to core
- **paths.rs** — new constants and helpers for `feedback-threads/` directory
- **error.rs** — one new variant `ThreadNotFound(String)`

The existing `feedback.rs` / `FeedbackNote` queue is untouched. Threads live at a separate path.

## Module: `sdlc-core/src/feedback_thread.rs`

### Structs

```rust
pub struct FeedbackThread {
    pub id: String,
    pub title: String,
    pub context: String,   // "feature:slug" | "ponder:slug" | free text
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub post_count: u32,
}

pub struct ThreadPost {
    pub seq: u32,
    pub author: String,    // "human" | "agent:<name>"
    pub content: String,
    pub created_at: DateTime<Utc>,
}
```

### Path helpers (in paths.rs)

```
feedback_threads_dir(root)                 → .sdlc/feedback-threads/
feedback_thread_dir(root, id)              → .sdlc/feedback-threads/<id>/
feedback_thread_manifest(root, id)         → .sdlc/feedback-threads/<id>/manifest.yaml
feedback_thread_posts_dir(root, id)        → .sdlc/feedback-threads/<id>/posts/
feedback_thread_post_path(root, id, seq)   → .sdlc/feedback-threads/<id>/posts/post-NNN.yaml
```

### Public functions

```rust
pub fn create_thread(root, context, title) -> Result<FeedbackThread>
pub fn load_thread(root, id) -> Result<FeedbackThread>
pub fn list_threads(root, filter_context: Option<&str>) -> Result<Vec<FeedbackThread>>
pub fn delete_thread(root, id) -> Result<()>
pub fn add_post(root, id, author, content) -> Result<ThreadPost>
pub fn list_posts(root, id) -> Result<Vec<ThreadPost>>
```

### ID generation

`<YYYYMMDD>-<sanitized-context>` — context sanitized by replacing `:/` with `-` and truncating to 55 chars (leaving room for date prefix). If a collision occurs, append `-2`, `-3`, etc.

### All writes use `io::atomic_write`. No `unwrap()` in library code.

## Module: `sdlc-cli/src/cmd/thread.rs`

Registered as `sdlc thread` in `cmd/mod.rs` and `main.rs`.

```
sdlc thread create <context> [--title <title>]
sdlc thread post   <id> --author <author> <content>
sdlc thread list   [--context <context>] [--json]
sdlc thread show   <id> [--json]
```

Table output via `print_table()`, JSON via `print_json()`.

## Module: `sdlc-server/src/routes/threads.rs`

Routes registered in `lib.rs`:

```
GET    /api/threads                  → list_threads (optional ?context= query param)
POST   /api/threads                  → create_thread
GET    /api/threads/:id              → get_thread  (thread + all posts inline)
POST   /api/threads/:id/posts        → add_post
DELETE /api/threads/:id              → delete_thread
```

All handlers use `tokio::task::spawn_blocking` wrapping core calls, consistent with `feedback.rs` pattern.

## JSON Schema

### Thread (list response)
```json
{
  "id": "20260302-feature-my-slug",
  "title": "Discussion: my-slug",
  "context": "feature:my-slug",
  "created_at": "2026-03-02T18:00:00Z",
  "updated_at": "2026-03-02T18:01:00Z",
  "post_count": 2
}
```

### Thread detail (GET /:id)
```json
{
  "id": "...",
  "title": "...",
  "context": "...",
  "created_at": "...",
  "updated_at": "...",
  "post_count": 2,
  "posts": [
    { "seq": 1, "author": "human", "content": "...", "created_at": "..." },
    { "seq": 2, "author": "agent:advisor", "content": "...", "created_at": "..." }
  ]
}
```

## Error Handling

- `ThreadNotFound(id)` → 404 Not Found
- Empty `author` or `content` on POST → 400 Bad Request
- All other errors → 500

## Testing Strategy

- Unit tests in `feedback_thread.rs` covering: create, load, list, add_post, list_posts, delete, collision-safe ID generation, context filter
- Route tests in `threads.rs` covering: list empty, create+get, add post, delete, 404 path
- All tests use `tempfile::TempDir` for isolation

## Files Changed

| File | Change |
|---|---|
| `crates/sdlc-core/src/feedback_thread.rs` | New |
| `crates/sdlc-core/src/lib.rs` | Add `pub mod feedback_thread` |
| `crates/sdlc-core/src/paths.rs` | Add constants + 5 path helpers |
| `crates/sdlc-core/src/error.rs` | Add `ThreadNotFound` variant |
| `crates/sdlc-cli/src/cmd/thread.rs` | New |
| `crates/sdlc-cli/src/cmd/mod.rs` | Register `thread` subcommand |
| `crates/sdlc-cli/src/main.rs` | Wire `thread` into CLI dispatch |
| `crates/sdlc-server/src/routes/threads.rs` | New |
| `crates/sdlc-server/src/routes/mod.rs` | Add `pub mod threads` |
| `crates/sdlc-server/src/lib.rs` | Register 5 routes |
