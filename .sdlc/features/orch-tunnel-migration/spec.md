# Spec: orch-tunnel migration — stable named URLs, default-on tunnel, graceful local fallback

## Problem

The current tunnel integration has three pain points:

1. **Cloudflare dependency**: `sdlc ui --tunnel` uses `cloudflared quick-tunnel`, which produces random hostnames (e.g. `fancy-rabbit-xyz.trycloudflare.com`) that change on every restart. Jordan does not control the Cloudflare edge.

2. **Opt-in friction**: Tunnel must be explicitly requested with `--tunnel`. Jordan wants tunnels to start automatically without extra flags.

3. **Hard failure mode**: If the tunnel binary is not found or fails to connect, `sdlc ui` errors out. In offline environments or when the edge is unreachable, there should be a graceful fallback to local-only mode.

## Goal

Migrate from cloudflared to orch-tunnel, make tunnel start by default, and fail gracefully if the tunnel cannot start.

## Scope

This feature covers three tightly related changes:

### 1. orch-tunnel as the only tunnel provider

Replace cloudflared with orch-tunnel in all tunnel code:
- `Tunnel::start()` spawns `orch-tunnel http <port> --name <project-name>` instead of cloudflared
- URL extraction reads `https://{name}.tunnel.threesix.ai` from orch-tunnel stdout
- Error messages reference `orch-tunnel` install instructions (not cloudflared)
- The binary lookup uses `which::which("orch-tunnel")`

Named tunnels produce a stable URL (`https://sdlc.tunnel.threesix.ai`) that does not change between restarts. This is a significant UX improvement over cloudflared's random hostnames.

**Status as of implementation start:** `crates/sdlc-server/src/tunnel.rs` already implements orch-tunnel. This part is complete.

### 2. Default-on tunnel

Change the CLI interface so tunnel starts by default:
- Remove `--tunnel` flag from `sdlc ui start`
- Add `--no-tunnel` flag to opt out
- If the user omits `--no-tunnel`, the tunnel starts automatically
- Warning message when tunnel mode activates: "Warning: tunnel mode exposes your SDLC server publicly."

### 3. Graceful local fallback

If orch-tunnel fails to start (binary not found, network unreachable, edge down):
- Log a warning to stderr: "Warning: orch-tunnel failed to start ({reason}). Running in local-only mode."
- Continue serving on localhost — do not return an error
- No change to authentication behavior — local access remains unauthenticated

## Out of Scope

- Named token auth (`.sdlc/auth.yaml`) — that is `auth-named-tokens`, a separate feature in the same milestone
- UI indicator for tunnel-unavailable state — terminal warning is sufficient for v1
- Token management CLI — deferred per ponder session decision

## Acceptance Criteria

1. `sdlc ui` (no flags) starts orch-tunnel automatically and prints the QR code
2. `sdlc ui --no-tunnel` starts without a tunnel
3. When orch-tunnel is not installed, `sdlc ui` logs a warning and continues in local-only mode
4. When orch-tunnel fails to connect, `sdlc ui` logs a warning and continues in local-only mode
5. Tunnel URL is stable across restarts (same project name → same URL)
6. All existing tests pass; new tests cover the graceful-fallback path
7. `--tunnel` flag is removed from the CLI help text; `--no-tunnel` appears instead

## Files Changed

| File | Change |
|------|--------|
| `crates/sdlc-server/src/tunnel.rs` | Already updated to orch-tunnel. No further changes needed. |
| `crates/sdlc-cli/src/cmd/ui.rs` | Flip `--tunnel` to `--no-tunnel`. Implement graceful fallback. |
| `crates/sdlc-cli/src/cmd/tunnel.rs` | `print_tunnel_info` call site — no structural change needed |

## Dependencies

- `auth-named-tokens` (sibling feature in v29-tunnel-auth) — not a blocker; these are independent changes
