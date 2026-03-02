# Security Audit: All state ops available over HTTP for remote consumers

## Scope

Two new HTTP endpoints:
- `POST /api/artifacts/:slug/:type/draft`
- `POST /api/features/:slug/merge`

## Authentication

Both endpoints are registered inside `build_router_from_state`, which applies the tunnel auth middleware to all routes:

```rust
.layer(axum::middleware::from_fn_with_state(
    app_state.tunnel_config.clone(),
    auth::auth_middleware,
))
```

The auth middleware enforces:
- Localhost/loopback requests bypass auth (local-only access)
- Tunnel requests require a valid session cookie or `auth` query parameter
- All other requests are blocked with 401

Neither new endpoint bypasses or modifies this middleware. Both are protected identically to every other `/api/*` route.

## Input Validation

### `POST /api/artifacts/:slug/:type/draft`

- `slug` path parameter — validated implicitly by `Feature::load`, which returns `FeatureNotFound` (→ 404) for unknown slugs. The slug format is not further validated here, consistent with all other artifact/feature endpoints.
- `artifact_type` path parameter — parsed through `ArtifactType::from_str` which rejects unknown types with `ArtifactNotFound` (→ 404). No injection vector.
- No request body — nothing to validate.

### `POST /api/features/:slug/merge`

- `slug` path parameter — validated by `Feature::load` (→ 404 if missing).
- Phase guard — `feature.phase != Phase::Merge` returns 400 before any state mutations occur.
- No request body — nothing to validate.

## State Mutation Analysis

### `draft_artifact`

Mutates: `feature.artifacts[n].status` → `draft`, `feature.updated_at`.

The `mark_artifact_draft` function does not write to disk directly; it only sets in-memory state. The subsequent `feature.save()` writes the manifest YAML atomically via `crate::io::atomic_write`. No other files are written.

This mutation is not destructive — it only advances artifact status. A repeated call is idempotent.

### `merge_feature`

Mutates: feature manifest phase field (`merge` → `released`), project state YAML (`active_directives`, `last_updated`).

Both writes go through atomic write helpers. The phase transition is gated on the current phase being `merge`. If the feature is already `released`, the phase guard (`target <= self.phase`) in `can_transition_to` would return `InvalidTransition` (→ 422) on the second call, so the endpoint is safe against double-merges.

## Path Traversal / Injection

Both endpoints use path parameters that are validated as slugs or enum variants before any filesystem access. The slug is used in `paths::feature_manifest(root, slug)` which constructs the path as `root/.sdlc/features/{slug}/manifest.yaml`. Slugs are validated by the `Feature::load` round-trip (which requires the directory to exist) and do not permit path traversal because the root is anchored to the configured SDLC root directory.

## No New Attack Surface

Neither endpoint:
- Executes shell commands
- Makes network requests
- Reads arbitrary file paths
- Exposes cryptographic material
- Accepts untrusted file content

## Verdict

APPROVED — no security concerns identified. Both endpoints follow the existing security posture of the server: protected by tunnel auth middleware, input validated through existing type-safe parsers, writes atomic and scope-limited to the SDLC root directory.
