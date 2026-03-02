# Spec: GET /api/features/:slug/directive

## Problem

Agents and tools that consume the sdlc HTTP API need a reliable, complete directive endpoint. The existing `GET /api/features/:slug/next` endpoint manually constructs its JSON response and omits the `description` field from the `Classification` struct. This creates a drift risk — any future field added to `Classification` also needs a manual addition in `features.rs`.

## Solution

Add `GET /api/features/:slug/directive` that serializes the full `Classification` struct via serde, identical in output to `sdlc next --for <slug> --json`.

## Endpoint Contract

**Request:**
```
GET /api/features/:slug/directive
```

**Response (200):**
Full `Classification` JSON matching the serde output of the struct, including:
- `feature` — feature slug
- `title` — feature title
- `description` — optional feature description (omitted when null, per `skip_serializing_if`)
- `current_phase` — current lifecycle phase
- `action` — next action type
- `message` — human-readable description of what to do
- `next_command` — slash command or CLI command to invoke
- `output_path` — optional path for artifact output
- `transition_to` — optional target phase
- `task_id` — optional task ID (for implement_task actions)
- `is_heavy` — advisory flag for resource-intensive actions
- `timeout_minutes` — advisory timeout budget

**Response (404):**
When the feature slug does not exist.

**Response (500):**
On classifier or I/O errors.

## Implementation Notes

- Route handler lives in `crates/sdlc-server/src/routes/features.rs`
- Use `Json(classification)` (axum JSON extractor serializing the `Classification` struct directly) rather than manually constructing `serde_json::json!{}`
- Register route in `crates/sdlc-server/src/lib.rs` alongside the `/next` route
- The existing `/next` endpoint is **not** changed — it remains for backward compatibility

## Out of Scope

- No changes to the `Classification` struct
- No changes to the CLI `sdlc next` command
- No changes to the existing `/next` endpoint
