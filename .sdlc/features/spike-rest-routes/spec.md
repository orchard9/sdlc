# Specification: Spike REST Routes

## Overview

Add REST API routes for spikes at `/api/spikes/*`. The routes expose the
existing `sdlc_core::spikes` data layer to the frontend and API consumers
with three endpoints: list, detail, and promote.

## Background

The spike data layer (`crates/sdlc-core/src/spikes.rs`) is fully implemented.
Spikes live at `.sdlc/spikes/<slug>/findings.md` and are written by the
`/sdlc-spike` agent. This feature adds the HTTP layer on top — thin wrappers
over the existing public API functions.

## Endpoints

### GET /api/spikes

Returns a JSON array of all spike entries, sorted by date descending.

**Response fields per entry:**
- `slug` (string) — directory name
- `title` (string) — parsed from `# Spike:` heading, or slug if absent
- `verdict` (string | null) — `"ADOPT"`, `"ADAPT"`, or `"REJECT"`
- `date` (string | null) — ISO date string from `**Date:**` field
- `the_question` (string | null) — content of `## The Question` section
- `ponder_slug` (string | null) — set after `promote_to_ponder` is called
- `knowledge_slug` (string | null) — set after `store_in_knowledge` is called

**Error cases:**
- `500` if the file system read fails

### GET /api/spikes/:slug

Returns a single spike's metadata plus the raw findings.md content.

**Response fields:**
- All fields from the list endpoint
- `findings` (string) — raw markdown content of findings.md (empty string if absent)

**Error cases:**
- `404` if the spike directory does not exist
- `500` for file system errors

### POST /api/spikes/:slug/promote

Promotes a spike to a ponder entry. Calls `sdlc_core::spikes::promote_to_ponder`.

**Request body (JSON, all optional):**
- `ponder_slug` (string | null) — override slug for the created ponder entry;
  defaults to the spike slug

**Response fields:**
- `ponder_slug` (string) — the slug of the created ponder entry

**Error cases:**
- `404` if the spike does not exist
- `422` if the spike verdict is not ADAPT (only ADAPT spikes should be promoted
  to active ponder work; ADOPT goes straight to implementation, REJECT goes to
  knowledge base)
- `500` for file system errors

## Implementation Notes

- New file: `crates/sdlc-server/src/routes/spikes.rs`
- Register module in `crates/sdlc-server/src/routes/mod.rs`
- Register routes in `crates/sdlc-server/src/lib.rs` following the same pattern
  as investigations and roadmap routes
- Use `axum::extract::{Path, State}`, `axum::Json`, `tokio::task::spawn_blocking`
- Map `SdlcError::Io(NotFound)` to `404`, other errors to `500` via `AppError`
- No auth middleware applied at the route level — existing server-wide auth
  middleware handles authentication as it does for all other `/api/*` routes
- No SSE events needed — these are read-only or simple write operations

## Out of Scope

- Frontend UI for spikes (separate feature)
- `store_in_knowledge` endpoint (REJECT spikes auto-file on list)
- POST to create a spike (spikes are created by the `/sdlc-spike` agent writing files directly)
