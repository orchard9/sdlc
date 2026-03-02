# Auth design — named tokens in auth.yaml

## Design: .sdlc/auth.yaml

```yaml
# .sdlc/auth.yaml
# Jordan manages this file directly. Add a token per person/device/agent.
# Revoking: delete the entry and save — hot-reloaded without server restart.
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
- If auth.yaml doesn't exist: generate one random token (backward-compatible), print to terminal
- `auth.yaml` should be in .gitignore (sensitive)

## What Jordan controls
- Who has access: add/remove token entries
- Per-device/per-agent naming: he knows which session is which
- Revocation: instant, no server restart
- Stable tokens: don't rotate on server restart

## What's NOT in v1
- bcrypt/hash storage (plain tokens are fine for personal tool)
- expiry dates
- sdlc auth add/revoke CLI (edit YAML directly)
- Audit log

## Files
- crates/sdlc-core/src/auth.rs (NEW) — AuthConfig struct, load/save, token validation
- crates/sdlc-server/src/auth.rs — update TunnelConfig → AuthConfig, hot-reload
- crates/sdlc-server/src/state.rs — add auth watcher