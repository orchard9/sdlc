# Design: Tunnel Preflight Check

## Overview

Three changes: a new server endpoint, a shared check function in `tunnel.rs`, and frontend gating in `NetworkPage.tsx`.

## Backend

### `check_orch_tunnel()` in `crates/sdlc-server/src/tunnel.rs`

New public function alongside `find_orch_tunnel()`:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckedLocation {
    pub location: String,
    pub found: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreflightResult {
    pub installed: bool,
    pub path: Option<String>,
    pub version: Option<String>,
    pub source: Option<String>,
    pub checked_locations: Vec<CheckedLocation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub install_hint: Option<String>,
}
```

Logic:
1. Try `which::which("orch-tunnel")` — record as `CheckedLocation { location: "process PATH", found }`.
2. Check well-known fallback paths (`/opt/homebrew/bin/orch-tunnel`, `/usr/local/bin/orch-tunnel`, `~/.cargo/bin/orch-tunnel`) — record each as a `CheckedLocation`.
3. If found via any method, run `<path> --version` (with 2s timeout) to capture version string.
4. Return `PreflightResult` with all fields populated.

This function is synchronous (blocking I/O for `which` and `Command::output`). The route handler wraps it in `spawn_blocking`.

### `GET /api/tunnel/preflight` in `crates/sdlc-server/src/routes/tunnel.rs`

```rust
pub async fn tunnel_preflight() -> Json<PreflightResult> {
    let result = tokio::task::spawn_blocking(check_orch_tunnel)
        .await
        .unwrap_or_else(|_| PreflightResult {
            installed: false,
            path: None,
            version: None,
            source: None,
            checked_locations: vec![],
            install_hint: Some(INSTALL_HINT.to_string()),
        });
    Json(result)
}
```

Route registration in `lib.rs`:
```rust
.route("/api/tunnel/preflight", get(routes::tunnel::tunnel_preflight))
```

No auth required — preflight is read-only diagnostic info.

## Frontend

### API client addition (`frontend/src/api/client.ts` or equivalent)

```typescript
getTunnelPreflight(): Promise<PreflightResult>
```

### NetworkPage.tsx changes

1. Both `SdlcTunnelSection` and `AppTunnelSection` call `api.getTunnelPreflight()` on mount.
2. Store result in `preflight` state.
3. When `preflight.installed === false`:
   - Disable the "Start tunnel" button.
   - Replace `<TunnelDisclosure>` with a `<PreflightWarning>` component showing:
     - Red/amber banner: "orch-tunnel not found"
     - Checked locations list
     - Install command with copy button
4. When `preflight.installed === true`:
   - Show version/path in `<TunnelDisclosure>` as subtle info.
   - Buttons enabled as before.

### UI states

See [Mockup](mockup.html) for visual reference.

| State | Button | Info area |
|---|---|---|
| Preflight loading | Disabled + spinner | "Checking orch-tunnel..." |
| Not installed | Disabled | Warning banner with install instructions |
| Installed, tunnel off | Enabled | Version + path shown subtly |
| Installed, tunnel on | Stop enabled | Tunnel URL, QR, etc. (unchanged) |

## Error formatting

No structural change needed — `TunnelError` display strings are already actionable. The frontend already renders them in the error banner. The preflight check prevents most "not found" errors from occurring at all.

## File change summary

| File | Change |
|---|---|
| `crates/sdlc-server/src/tunnel.rs` | Add `PreflightResult`, `CheckedLocation`, `check_orch_tunnel()` |
| `crates/sdlc-server/src/routes/tunnel.rs` | Add `tunnel_preflight` handler |
| `crates/sdlc-server/src/lib.rs` | Register `/api/tunnel/preflight` route |
| `frontend/src/api/client.ts` | Add `getTunnelPreflight()` |
| `frontend/src/lib/types.ts` | Add `PreflightResult` type |
| `frontend/src/pages/NetworkPage.tsx` | Add preflight check, `PreflightWarning` component, button gating |
