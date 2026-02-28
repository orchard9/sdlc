# Tunnel Gateway — Remote Access via QR Code

Expose the local `sdlc ui` server to the internet through a temporary authenticated tunnel, shareable via a QR code printed in the terminal.

---

## Problem

`sdlc ui` binds to `0.0.0.0:<port>` — accessible on the local machine only. There is no way to access the UI from a phone, tablet, or remote machine without manual network configuration. This makes remote review, mobile testing, and quick sharing with collaborators unnecessarily painful.

---

## Solution

Add a `--tunnel` flag to `sdlc ui start` that:

1. Starts the server on a random free port
2. Spawns a `cloudflared` quick-tunnel to get a public HTTPS URL
3. Generates a short random passcode and embeds it in the tunnel URL
4. Prints the QR code and passcode to the terminal
5. Enforces the passcode via Axum middleware on all requests through the tunnel

Local access (`localhost`) bypasses auth entirely — no disruption to normal workflow.

---

## User Experience

```
$ sdlc ui --tunnel

SDLC UI for 'my-project'
  Local:   http://localhost:52341  (no auth)
  Tunnel:  https://fancy-rabbit-deluxe.trycloudflare.com

  ┌─────────────────────────────────┐
  │                                 │
  │  █▀▀▀▀▀█ ▄▄▄▄▄ █ █▀▀▀▀▀█     │
  │  █ ███ █ █   █ ▀ █ ███ █     │
  │  █ ▀▀▀ █ █▄▄▄█   █ ▀▀▀ █     │
  │  ▀▀▀▀▀▀▀ ▀ ▀ ▀   ▀▀▀▀▀▀▀     │
  │  ...                            │
  │                                 │
  └─────────────────────────────────┘

  Passcode:  K7mX9pQ2
  (embedded in QR — scan to access)

Ctrl+C to stop tunnel and server
```

Scanning the QR code opens `https://fancy-rabbit-deluxe.trycloudflare.com/?auth=K7mX9pQ2`. The middleware sets a session cookie and redirects to `/`. Subsequent requests use the cookie — no token in the URL after first visit.

---

## Architecture

### Components

```
sdlc ui --tunnel
    │
    ├── TcpListener (random port)
    ├── Axum server  ←── AuthMiddleware (token or cookie)
    │                         ↑
    │                   LocalBypass (skip if Host == localhost)
    │
    └── cloudflared tunnel process
            └── https://<random>.trycloudflare.com → localhost:<port>
```

### Auth Middleware

All requests through the tunnel require a valid session:

- **Cookie `sdlc_auth`** matches the token → allow
- **Query param `?auth=TOKEN`** matches → set cookie, redirect to path without query
- **Neither** → 401 with JSON `{"error": "unauthorized"}` for `/api/*`, or minimal HTML "Access denied — scan the QR code" for all other paths
- **Local bypass** — if `Host` header is `localhost` or `127.0.0.1`, skip auth entirely

### Token Generation

8-character alphanumeric token using `rand`. Sufficient entropy (~47 bits) for a temporary dev tunnel. Regenerated on every `sdlc ui --tunnel` invocation — not persisted.

### Tunnel Backend

`cloudflared` is the primary and only supported backend for v1:

```bash
cloudflared tunnel --url http://localhost:<PORT> --no-autoupdate
```

The public HTTPS URL is parsed from `cloudflared` stderr output — it logs a line like:
```
Your quick Tunnel has been created! Visit it at (it may take some time to be reachable):
https://fancy-rabbit-deluxe.trycloudflare.com
```

We capture this with a regex on stderr before printing the QR code.

#### Graceful degradation

If `cloudflared` is not in PATH, print a clear error:

```
✗ cloudflared not found.

  Install it with:
    brew install cloudflare/cloudflare/cloudflared   # macOS
    winget install Cloudflare.cloudflared             # Windows
    curl -L ... | sh                                  # Linux

  Then re-run: sdlc ui --tunnel
```

No silent fallback — be explicit.

---

## Implementation Plan

### New files

| File | Purpose |
|---|---|
| `crates/sdlc-cli/src/cmd/tunnel.rs` | Tunnel process management, URL extraction, QR rendering |

### Modified files

| File | Change |
|---|---|
| `crates/sdlc-cli/src/cmd/ui.rs` | Add `--tunnel` flag; wire tunnel startup + teardown |
| `crates/sdlc-server/src/lib.rs` | Accept optional `TunnelConfig`; add auth middleware |
| `crates/sdlc-server/src/auth.rs` | New: Axum middleware — cookie/token check + local bypass |
| `crates/sdlc-cli/Cargo.toml` | Add `qrcode`, confirm `rand` present |
| `crates/sdlc-server/Cargo.toml` | No new deps (cookie handling via raw axum headers) |

### New dependencies

| Crate | Version | Purpose |
|---|---|---|
| `qrcode` | `0.14` | Terminal QR code rendering (pure Rust) |
| `rand` | `0.8` (already in workspace) | Token generation |

---

## Security Properties

| Property | Value |
|---|---|
| Transport | HTTPS via Cloudflare (TLS terminated at edge) |
| Auth mechanism | HMAC-free shared token; session cookie |
| Token entropy | ~47 bits (8 alphanumeric chars from 62-char alphabet) |
| Token lifetime | Single process lifetime — ephemeral |
| Local access | Unauthenticated (localhost only) |
| Brute force | Impractical — URL is also unknown; Cloudflare rate-limits |
| Threat model | Casual snooping, accidental exposure — not nation-state |

This is appropriate for a developer tool used in short sessions. It is **not** suitable for long-running production exposure.

---

## Out of Scope (v1)

- Multiple tunnel backends (bore, serveo, localhost.run)
- Custom passcode (user-specified token)
- Persistent tunnel URLs (paid Cloudflare Tunnels)
- Per-route access control
- Audit log of tunnel access
- `sdlc ui list` showing tunnel URL alongside local URL (v2)

---

## Acceptance Criteria

- `sdlc ui --tunnel` starts server, spawns tunnel, prints QR code and passcode
- QR code URL opens the app after one scan (cookie set, no second auth prompt)
- Accessing tunnel URL without token returns 401
- Local `http://localhost:<port>` works without any auth
- `cloudflared` not in PATH prints a clear install message and exits cleanly
- Ctrl+C kills both the tunnel process and the server cleanly
- `SDLC_NO_NPM=1 cargo test --all` passes (no test regressions)
