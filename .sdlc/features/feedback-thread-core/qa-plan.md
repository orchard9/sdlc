# QA Plan: FeedbackThread core — data model, CLI, and REST API

## QA Approach

All quality verification is automated. No manual steps required.

## Test Suite: sdlc-core (unit tests in feedback_thread.rs)

| ID | Test | Pass Criterion |
|----|------|---------------|
| U1 | `create_thread` stores manifest with correct fields | `load_thread` returns same context, title, post_count=0 |
| U2 | `list_threads` empty when no threads exist | Returns empty Vec |
| U3 | `list_threads` with context filter returns only matching threads | Two threads created, filter returns one |
| U4 | `add_post` increments seq and updates post_count | First post has seq=1, second has seq=2 |
| U5 | `list_posts` returns posts ordered by seq ascending | Two posts inserted, verify order |
| U6 | `delete_thread` removes directory | Thread dir does not exist after delete |
| U7 | `ThreadNotFound` returned when loading nonexistent thread | Err matches `SdlcError::ThreadNotFound` |
| U8 | ID generation is collision-safe | Create two threads with same context, IDs differ |
| U9 | Context with special chars (`:/`) sanitizes to valid path segment | ID contains no colons or slashes |
| U10 | `load_thread` on empty posts dir returns zero posts via `list_posts` | Zero posts returned |

## Test Suite: sdlc-server (route tests in threads.rs)

| ID | Test | Pass Criterion |
|----|------|---------------|
| R1 | `GET /api/threads` returns empty array initially | Status 200, body `[]` |
| R2 | `POST /api/threads` creates thread, response has `id` and `context` | Status 200, fields present |
| R3 | `GET /api/threads/:id` returns thread with empty `posts` array | Status 200, `posts: []` |
| R4 | `POST /api/threads/:id/posts` appends post, response has updated `posts` array | Status 200, posts.len() == 1 |
| R5 | `DELETE /api/threads/:id` returns `{ deleted: true }` | Status 200 |
| R6 | `GET /api/threads/:id` on deleted thread returns 404 | Status 404 |
| R7 | `POST /api/threads/:id/posts` with empty content returns 400 | Status 400 |
| R8 | `POST /api/threads/:id/posts` with empty author returns 400 | Status 400 |
| R9 | `GET /api/threads?context=feature:x` filters threads | Only matching thread returned |
| R10 | `GET /api/threads/:nonexistent` returns 404 | Status 404 |

## Test Suite: sdlc-cli (integration via tempdir)

Covered implicitly by core unit tests. CLI dispatch correctness is verified via `cargo test` in sdlc-cli; no additional CLI integration tests required for this feature tier.

## Build Gates

```bash
SDLC_NO_NPM=1 cargo test --all     # all unit and route tests pass
cargo clippy --all -- -D warnings  # zero warnings
```

Both commands must exit 0.

## Out of Scope

- Performance benchmarks
- Load testing
- Browser automation (UI covered by feedback-thread-ui)
