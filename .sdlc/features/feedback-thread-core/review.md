# Code Review: FeedbackThread core — data model, CLI, and REST API

## Summary

Implementation is complete. All 9 tasks delivered. 28 tests pass (17 core unit + 11 server route tests). Zero clippy warnings. Build is clean.

## Files Changed

| File | Change | Lines |
|---|---|---|
| `crates/sdlc-core/src/paths.rs` | +1 constant, +5 path helpers, +10 test assertions | +47 |
| `crates/sdlc-core/src/error.rs` | +1 error variant `ThreadNotFound` | +3 |
| `crates/sdlc-core/src/feedback_thread.rs` | New — full data module | +406 |
| `crates/sdlc-core/src/lib.rs` | +1 module declaration | +1 |
| `crates/sdlc-server/src/error.rs` | +1 match arm for `ThreadNotFound → 404` | +1 |
| `crates/sdlc-cli/src/cmd/thread.rs` | New — CLI subcommand | +156 |
| `crates/sdlc-cli/src/cmd/mod.rs` | +1 module declaration | +1 |
| `crates/sdlc-cli/src/main.rs` | +1 import, +1 enum variant, +1 dispatch arm | +6 |
| `crates/sdlc-server/src/routes/threads.rs` | New — REST routes | +395 |
| `crates/sdlc-server/src/routes/mod.rs` | +1 module declaration | +1 |
| `crates/sdlc-server/src/lib.rs` | +5 route registrations | +9 |

## Correctness

- All CRUD operations verified: create, load, list (with filter), add_post, list_posts, delete.
- `ThreadNotFound` returned correctly for load, add_post, list_posts, delete on nonexistent IDs.
- Collision-safe ID generation verified — two threads with the same context on the same day get different IDs.
- Context sanitization strips `:`, `/`, spaces, and collapses runs of dashes. Verified no colons/slashes in generated IDs.
- `post_count` and `updated_at` on the thread manifest are updated atomically on each `add_post` call.
- Posts are sorted by `seq` ascending on `list_posts` — filesystem traversal order is not assumed.

## Spec Compliance

All acceptance criteria from spec.md are met:
1. `sdlc thread create "feature:my-slug"` creates manifest — confirmed by `create_and_load_thread` test
2. `sdlc thread post <id> --author human "text"` appends post-001.yaml with seq=1 — confirmed by `add_posts_increments_seq_and_post_count`
3. `sdlc thread show <id>` prints manifest + posts — implemented in CLI `show` branch
4. `GET /api/threads?context=feature:...` returns matching thread — confirmed by `list_with_context_filter` route test
5. `POST /api/threads/:id/posts` appends and returns updated thread — confirmed by `add_post_appends_and_returns_thread_with_posts`
6. `SDLC_NO_NPM=1 cargo test --all` passes — confirmed

## Code Quality

- No `unwrap()` in library code — all fallible operations return `Result`
- All file writes go through `io::atomic_write` — consistent with the rest of the codebase
- Route handlers use `spawn_blocking` — consistent with `feedback.rs` and all other I/O routes
- Validation in server routes: empty `context`, `author`, `content` all return 400 before hitting core
- `ThreadNotFound` wired into `error.rs` match arm — returns 404 correctly

## Design Adherence

- Storage layout matches design: `.sdlc/feedback-threads/<id>/manifest.yaml` + `posts/post-NNN.yaml`
- JSON response schema matches design: thread fields + `posts` array inline on GET /:id
- `add_post` route returns `new_post` field alongside the full updated thread — useful for UI consumers
- CLI follows the `print_table` / `print_json` output convention used throughout the CLI

## Findings

No blocking issues found. One non-blocking note:

- **Note (non-blocking):** The `add_post` route currently returns the full thread + all posts + `new_post` in a single response. This is slightly over-specified but useful for the UI feature (`feedback-thread-ui`) that will be built next. No change needed.

## Verdict

APPROVED — implementation is complete, correct, spec-compliant, and consistent with codebase patterns.
