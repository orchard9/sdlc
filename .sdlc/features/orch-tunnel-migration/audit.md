# Security Audit: orch-tunnel migration

## Scope

This audit covers the security implications of:
1. Replacing cloudflared with orch-tunnel as the tunnel provider
2. Making tunnel default-on (`--no-tunnel` to opt out)
3. Implementing graceful fallback when orch-tunnel fails

---

## Security Surface Analysis

### 1. Tunnel provider change (cloudflared → orch-tunnel)

**Before:** cloudflared quick-tunnels expose the server at a random `*.trycloudflare.com` URL. Cloudflare terminates TLS and forwards HTTP to the local port. Token auth is enforced by the sdlc auth middleware.

**After:** orch-tunnel exposes the server at a stable `<name>.tunnel.threesix.ai` URL. Jordan controls the edge server at `connect.tunnel.threesix.ai:7844`. TLS termination happens at the edge. Token auth is enforced by the sdlc auth middleware.

**Security impact:**
- The auth middleware is unchanged — no auth bypass introduced
- URL stability (same URL on restart) is a minor security improvement: Jordan can better track who has the URL and revoke tokens without the URL changing
- Jordan controls the edge server, which means the trust relationship shifts from Cloudflare (third party) to Jordan's own infrastructure. This is a positive change for control, but it means Jordan's orch-tunnel server uptime is now a security dependency. If the edge goes down, the fallback is local-only (not public exposure), so there is no risk of an unsafe fallback.
- No new external network calls or credential exposure introduced

**Finding: NONE**

### 2. Default-on tunnel

**Before:** Tunnel requires explicit `--tunnel` flag. Default is local-only (no public exposure).

**After:** Tunnel starts by default. Explicit `--no-tunnel` to disable.

**Security impact:**
- This is a change in default exposure policy. Running `sdlc ui` without flags now starts a public tunnel.
- A warning is printed: `"Warning: tunnel mode exposes your SDLC server publicly. Share the QR code only with trusted parties."`
- Auth is still enforced on the tunnel: the `?auth=TOKEN` bootstrap flow and cookie auth are unchanged
- An operator who was previously relying on "no --tunnel = no public exposure" will now have public exposure by default unless they explicitly pass `--no-tunnel`

**Finding:** This is a user-visible security behavior change. The warning message is present but it's printed at start time — operators in automated/daemon contexts may not see it. This is acceptable for a developer tool with single-user deployment, but should be noted.

**Action:** Document explicitly in `docs/orch-tunnel-reference.md` (done in T3). No code change required — the behavior is intentional per the spec.

### 3. Graceful fallback

**Before:** Tunnel failure exits the process. Local-only mode requires `--no-tunnel`.

**After:** Tunnel failure logs a warning and continues in local-only mode.

**Security impact:**
- The fallback path drops to local-only (no public exposure). This is safer than the alternative (error exit).
- The registry record is not removed on tunnel failure — the server keeps running on localhost. This is correct.
- An operator relying on "tunnel failure = no server running" will now have a server running locally when the tunnel fails. For a developer tool, this is acceptable.

**Finding:** Behavior change is safe (local-only fallback is less exposure than tunnel, not more). No security concern.

### 4. Token generation

The `generate_token()` function in `sdlc_server::tunnel` is unchanged — still produces a random 8-character alphanumeric token using `rand::thread_rng()`. This is sufficient for a developer tool with single-user access.

**Finding: NONE**

### 5. Auth middleware

The `auth_middleware` in `sdlc-server/src/auth.rs` is not modified by this feature. All auth logic (localhost bypass, cookie validation, `?auth=TOKEN` bootstrap, `/__sdlc/` public path) is unchanged.

The `TunnelConfig` struct was already changed by the `auth-named-tokens` working tree changes from `token: Option<String>` to `tokens: Vec<(String, String)>` before this feature ran. The `routes/tunnel.rs` fix in this feature corrects a compile error caused by that pre-existing change — it removes the stale token field access from `get_tunnel`, returning `None` for the token on GET. This is actually more secure (tokens are no longer readable via GET after initial generation).

**Finding:** The GET /api/tunnel no longer leaks the active token — this is an improvement.

---

## Summary

| Finding | Severity | Action |
|---------|----------|--------|
| Default-on tunnel changes exposure defaults | Info | Documented in reference docs (done) |
| GET /api/tunnel no longer exposes token | Improvement | Addressed in this feature (tunnel.rs fix) |

No security blockers. The feature is safe to ship.
