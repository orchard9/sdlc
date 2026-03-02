# Tasks: FeedbackThread core — data model, CLI, and REST API

## T1 — Add path constants and helpers to paths.rs

Add `FEEDBACK_THREADS_DIR` constant and helper functions:
- `feedback_threads_dir(root)` → `.sdlc/feedback-threads/`
- `feedback_thread_dir(root, id)` → `.sdlc/feedback-threads/<id>/`
- `feedback_thread_manifest(root, id)` → `.sdlc/feedback-threads/<id>/manifest.yaml`
- `feedback_thread_posts_dir(root, id)` → `.sdlc/feedback-threads/<id>/posts/`
- `feedback_thread_post_path(root, id, seq)` → `.sdlc/feedback-threads/<id>/posts/post-NNN.yaml`

Also add unit tests for the new helpers in the existing `paths::tests` block.

File: `crates/sdlc-core/src/paths.rs`

## T2 — Add ThreadNotFound error variant

Add `#[error("thread not found: {0}")] ThreadNotFound(String)` to `SdlcError` in `error.rs`.

File: `crates/sdlc-core/src/error.rs`

## T3 — Implement feedback_thread.rs (core data module)

Create `crates/sdlc-core/src/feedback_thread.rs` with:

**Structs** (both `Serialize + Deserialize + Clone + Debug`):
- `FeedbackThread { id, title, context, created_at, updated_at, post_count }`
- `ThreadPost { seq, author, content, created_at }`

**Private helpers**:
- `load_thread_manifest(root, id) -> Result<FeedbackThread>`
- `save_thread_manifest(root, thread) -> Result<()>`
- `make_thread_id(root, context) -> String` — date-prefix + sanitized context, collision-safe

**Public API**:
- `create_thread(root, context, title) -> Result<FeedbackThread>`
- `load_thread(root, id) -> Result<FeedbackThread>`
- `list_threads(root, filter_context: Option<&str>) -> Result<Vec<FeedbackThread>>`
- `delete_thread(root, id) -> Result<()>`
- `add_post(root, id, author, content) -> Result<ThreadPost>`
- `list_posts(root, id) -> Result<Vec<ThreadPost>>`

All writes via `io::atomic_write`. No `unwrap()`.

**Unit tests** using `tempfile::TempDir`:
- create and load
- list empty / list with filter
- add posts, verify seq increments
- delete cleans up directory
- collision-safe ID generation

## T4 — Register module in sdlc-core lib.rs

Add `pub mod feedback_thread;` to `crates/sdlc-core/src/lib.rs`.

File: `crates/sdlc-core/src/lib.rs`

## T5 — Implement sdlc-cli thread subcommand

Create `crates/sdlc-cli/src/cmd/thread.rs` with clap-derived subcommands:

```
sdlc thread create <context> [--title <title>]
sdlc thread post   <id> --author <author> <content>
sdlc thread list   [--context <context>] [--json]
sdlc thread show   <id> [--json]
```

- `create`: prints new thread ID
- `post`: prints seq number
- `list`: table with columns ID | context | post_count | updated_at (or JSON)
- `show`: prints manifest fields then all posts (or JSON)

Use `print_table()` and `print_json()` from output.rs.

File: `crates/sdlc-cli/src/cmd/thread.rs`

## T6 — Register thread subcommand in CLI mod.rs and main.rs

Wire `thread` into the top-level CLI dispatch:
- Add `pub mod thread;` to `crates/sdlc-cli/src/cmd/mod.rs`
- Add `Commands::Thread(args) => cmd::thread::run(root, args)` to `crates/sdlc-cli/src/main.rs`
- Add the `Thread` variant with its `ThreadArgs` to the `Commands` enum in main.rs

Files: `crates/sdlc-cli/src/cmd/mod.rs`, `crates/sdlc-cli/src/main.rs`

## T7 — Implement sdlc-server routes/threads.rs

Create `crates/sdlc-server/src/routes/threads.rs` with:

- `list_threads(State, Query<ListQuery>) -> JSON` — GET /api/threads
- `create_thread(State, Json<CreateBody>) -> JSON` — POST /api/threads
- `get_thread(State, Path<id>) -> JSON` — GET /api/threads/:id (returns thread + posts)
- `add_post(State, Path<id>, Json<PostBody>) -> JSON` — POST /api/threads/:id/posts
- `delete_thread(State, Path<id>) -> JSON` — DELETE /api/threads/:id

All handlers use `spawn_blocking`. Validate non-empty `author` and `content` with 400. Return 404 on `ThreadNotFound`.

Include route-level unit tests covering: list empty, create+get, add post, delete, 404 cases.

File: `crates/sdlc-server/src/routes/threads.rs`

## T8 — Register routes in server mod.rs and lib.rs

- Add `pub mod threads;` to `crates/sdlc-server/src/routes/mod.rs`
- Register all 5 routes in `crates/sdlc-server/src/lib.rs` under the auth layer

Files: `crates/sdlc-server/src/routes/mod.rs`, `crates/sdlc-server/src/lib.rs`

## T9 — Verify build and tests

Run:
```bash
SDLC_NO_NPM=1 cargo test --all
cargo clippy --all -- -D warnings
```

Fix any warnings or test failures before marking done.
