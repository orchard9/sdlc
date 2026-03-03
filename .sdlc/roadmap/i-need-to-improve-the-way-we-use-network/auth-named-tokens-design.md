# Auth design — named tokens in auth.yaml

## Design: .sdlc/auth.yaml

```yaml
# .sdlc/auth.yaml
# Jordan manages this file directly. Add a token per person/device/agent.
# Revoking: delete the entry and save — hot-reloaded without server restart.
# If this file is absent or auth.enabled is false, server runs open (no auth required).
auth:
  enabled: true
tokens:
  - name: jordan-phone
    token: "abc123def456"
  - name: ci-agent
    token: "xyz987mno234"
  - name: guest-review
    token: "def456abc789"
```

## Behavior
- Server reads auth.yaml on startup
- mtime watcher hot-reloads when file changes (existing infrastructure in state.rs)
- Any token in the list grants access
- Cookie key stays `sdlc_auth`, value is the matching token
- `?auth=<token>` bootstrap flow unchanged
- **If auth.yaml doesn't exist OR auth.enabled is false → server runs open, no auth required**
  - This replaces the previous 'generate random token' fallback — auth is truly optional
- `auth.yaml` should be in .gitignore (sensitive)

## Auth is optional and easily viewable (confirmed Session 2)
- Optional: absence of auth.yaml or `auth: {enabled: false}` = no auth
- Easily viewable: plain tokens in YAML (human-readable, no hashing)
- UI: settings/sidebar panel shows token names + enabled status (NOT token values)
  - Minimal: list of `name` fields with a green/grey status indicator
  - Makes it easy to see who has access without opening the file

## What Jordan controls
- Who has access: add/remove token entries
- Per-device/per-agent naming: he knows which session is which
- Revocation: instant, no server restart
- Stable tokens: don't rotate on server restart
- Enable/disable entirely: flip `auth.enabled` or delete file

## What's NOT in v1
- bcrypt/hash storage (plain tokens are fine for personal tool)
- expiry dates
- `sdlc auth add/revoke` CLI (edit YAML directly)
- Audit log / session tracking

## Files
- `crates/sdlc-core/src/auth.rs` (NEW) — AuthConfig struct, load/save, token validation
- `crates/sdlc-server/src/auth.rs` — update TunnelConfig → AuthConfig, hot-reload, optional auth
- `crates/sdlc-server/src/state.rs` — add auth watcher
- `frontend/src/components/layout/QuotaPanel.tsx` (or new `AuthPanel.tsx`) — show token names + enabled status