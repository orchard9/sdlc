# Spec: FeedbackThread core — data model, CLI, and REST API

## Problem

The current feedback system is a flat global queue (`FeedbackNote` in `feedback.rs`). Notes have no context anchor — they are just text in a list. There is no way to attach discussion to a specific feature, ponder entry, or knowledge entry. Humans and agents cannot exchange structured, identified, contextual comments on a live topic.

## Goal

Introduce `FeedbackThread` as a first-class persistent primitive. A thread is a lightweight comment log anchored to a named subject (identified by a `context` string like `"feature:my-slug"`, `"ponder:my-idea"`, or `"knowledge:entry-slug"`). Each thread holds an append-only list of `ThreadPost` entries authored by either a human or an agent. Threads are stored as YAML files under `.sdlc/feedback-threads/`.

The existing `FeedbackNote` queue is not changed or removed — threads are a new data type alongside it.

## Scope

- **sdlc-core**: new `feedback_thread.rs` module with structs and CRUD functions
- **sdlc-cli**: new `sdlc thread` subcommand group (create, post, list, show)
- **sdlc-server**: new REST routes at `/api/threads/*`
- **paths.rs**: new path helpers for the `feedback-threads` directory
- **error.rs**: new `ThreadNotFound` error variant
- **lib.rs** (core): expose new module
- **lib.rs** (server): register new routes

## Data Model

### `FeedbackThread`
```yaml
id: "20260302-feature-my-slug"
title: "Discussion: my-slug"
context: "feature:my-slug"   # arbitrary dot-or-colon namespaced string
created_at: "2026-03-02T18:00:00Z"
updated_at: "2026-03-02T18:00:00Z"
post_count: 2
```

Storage: `.sdlc/feedback-threads/<id>/manifest.yaml`

### `ThreadPost`
```yaml
seq: 1
author: "human"          # "human" | "agent:<name>"
content: "This approach worries me."
created_at: "2026-03-02T18:01:00Z"
```

Storage: `.sdlc/feedback-threads/<id>/posts/post-NNN.yaml`

IDs are generated as `<YYYYMMDD>-<context_slug>` where `context_slug` is the context string with `:` and `/` replaced by `-`, truncated to fit the 64-char slug limit.

## CLI

```
sdlc thread create <context> [--title <title>]
sdlc thread post   <id> --author <author> <content>
sdlc thread list   [--context <context>]
sdlc thread show   <id>
```

- `create` prints the new thread ID
- `post` appends a post and prints the post sequence number
- `list` prints a table: ID | title | context | post_count | updated_at
- `show` prints the thread manifest followed by all posts in order

## REST API

```
GET    /api/threads                  — list all threads (optional ?context= filter)
POST   /api/threads                  — create thread { context, title? }
GET    /api/threads/:id              — get thread + posts
POST   /api/threads/:id/posts        — append post { author, content }
DELETE /api/threads/:id              — delete thread (returns { deleted: true })
```

All responses are JSON. Posts are returned inline in the GET /:id response.

## Storage Layout

```
.sdlc/
  feedback-threads/
    20260302-feature-my-slug/
      manifest.yaml
      posts/
        post-001.yaml
        post-002.yaml
```

## Acceptance Criteria

1. `sdlc thread create "feature:my-slug"` creates `.sdlc/feedback-threads/<id>/manifest.yaml`
2. `sdlc thread post <id> --author human "text"` appends a post file with seq=1
3. `sdlc thread show <id>` prints the manifest and both posts in order
4. `GET /api/threads?context=feature:my-slug` returns the created thread
5. `POST /api/threads/:id/posts` appends a post and returns the updated thread
6. All unit tests pass with `SDLC_NO_NPM=1 cargo test --all`

## Out of Scope

- UI components (covered by `feedback-thread-ui`)
- SSE events for thread updates
- Editing or deleting individual posts (append-only by design)
- Thread search or full-text indexing
