# Security Audit: Spike REST Routes

## Scope

Three new HTTP endpoints:
- `GET /api/spikes`
- `GET /api/spikes/:slug`
- `POST /api/spikes/:slug/promote`

## Authentication and Authorization

**Finding:** All three routes are registered in `build_router_from_state` before
the `auth_middleware` layer at line 646 of `lib.rs`. Every route registered in that
function sits behind the same auth layer — the middleware is applied via
`.layer(axum::middleware::from_fn_with_state(...auth_middleware))`. The spikes
routes are no exception.

**Status:** No vulnerability. Auth coverage is inherited from the server's unified
auth layer. No route-level bypass is possible.

## Input Validation

**Slug parameter (`GET /api/spikes/:slug`, `POST /api/spikes/:slug/promote`):**
- `sdlc_core::spikes::load` calls `crate::paths::validate_slug(slug)?` before any
  file system access. Invalid slugs (path traversal characters, `..`, `/`) are
  rejected with an error before any file read occurs.
- Axum extracts the slug from the URL path via `Path(slug): Path<String>`. The
  slug is a URL path segment so `/` characters are not passable as part of a slug.

**Promote body (`POST /api/spikes/:slug/promote`):**
- Body is `Option<Json<PromoteBody>>` with only one optional string field
  (`ponder_slug`). If provided, it is passed to `promote_to_ponder` which also
  calls `validate_slug` on the override slug.
- No injection surface — the string is used only as a directory name after
  validation.

**Status:** No vulnerability.

## Path Traversal

- All spike file access is routed through `sdlc_core::paths::spike_findings_path`
  and `spike_state_path` which construct paths as `root.join(SPIKES_DIR).join(slug)`.
- `validate_slug` rejects slugs containing path separators or `..` sequences,
  preventing traversal outside the `.sdlc/spikes/` directory.

**Status:** No vulnerability.

## Side Effects

- `list` auto-files REJECT spikes into the knowledge base. This is a pre-existing
  data-layer behavior, not new. The HTTP call does not give callers control over
  which spikes are filed.
- `promote_to_ponder` creates files under `.sdlc/roadmap/`. The created directory
  name is either the spike slug or a caller-supplied override — both validated.

**Status:** Acceptable. Side effects are bounded to local state in `.sdlc/`.

## Information Disclosure

- The `findings` field in `GET /api/spikes/:slug` returns the raw markdown content
  of `findings.md`. This is internal project documentation — the same information
  available to anyone with file system access. No credentials or secrets are
  expected in findings.md by convention.
- Error responses do not expose stack traces or internal paths beyond the error
  message already present in `SdlcError`.

**Status:** Acceptable. Content classification is consistent with other routes
(e.g., knowledge, investigations) that also return raw artifact content.

## Denial of Service

- `list` reads all spike directories and their findings files. With a large number
  of spikes this could be slow, but it is bounded by the number of directories under
  `.sdlc/spikes/` which is a developer-controlled local directory. No unbounded
  input from the network.

**Status:** Acceptable for this deployment context (single-tenant local server).

## Summary

No security issues found. All findings are acceptable or non-applicable. The
implementation correctly inherits server-wide authentication and performs slug
validation before any file I/O.
