---
session: 2
timestamp: 2026-03-02T23:55:00Z
orientation:
  current: "Commit signal met. Jordan confirmed all three design decisions: orch-tunnel swap, default-on tunnel, named token auth. Auth is optional (no auth.yaml = open access) and easily viewable (plain YAML + UI token name list). Ready to commit to a milestone."
  next: "/sdlc-ponder-commit i-need-to-improve-the-way-we-use-network — scope into one milestone with three features: orch-tunnel-swap, tunnel-auto-start, named-token-auth"
  commit: "MET — all design decisions confirmed by Jordan."
---

**jordan · Owner**
yes that works - auth should be optional and easily viewable
yes auto creating makes sense - all looks good

---

## Session 2 — Commit Signal Achieved

**Facilitator note:** Jordan's message closes all three open questions from Session 1. This is a confirmation pass, not a new design session. Let me read what changed and finalize the picture.

---

### Reading Jordan's confirmation

Session 1 left three open questions:
1. `?` Should tokens live in `auth.yaml` (separate) vs `config.yaml`?
2. `?` Should the UI show "tunnel unavailable" when local-only mode triggers?
3. `?` Is edge-layer auth at orch-tunnel server in scope?

Jordan's message answers them:

> "auth should be optional and easily viewable"
> "yes auto creating makes sense - all looks good"

**Dan Reeves:** "Optional and easily viewable" does a lot of work. Let me parse it precisely:
- **Optional** = auth can be disabled entirely. The previous design had a fallback "generate random token if auth.yaml absent." That's wrong — optional means absent = open. Jordan decides whether to turn it on.
- **Easily viewable** = the token list is visible without digging. Plain YAML (no bcrypt) satisfies this for the file. But "viewable" in the context of a UI tool also implies: the UI shows what's active. A small auth status section in the sidebar (token names, enabled/disabled) completes this.

⚑ **Decided:** If `auth.yaml` is absent or has `auth: {enabled: false}`, server runs open — no auth required. This supersedes the "generate random token" fallback.

⚑ **Decided:** Auth tokens are plain strings in YAML (not hashed). "Easily viewable" is a requirement, bcrypt would violate it.

⚑ **Decided:** UI shows token names (not values) + enabled status. Minimal list in settings/sidebar panel.

**Leila Hassan:** The "easily viewable" UI requirement is small but meaningful. In the QuotaPanel area or a dedicated `AuthPanel`, showing:

```
Auth: enabled
• jordan-phone ✓
• ci-agent ✓
• guest-review ✓
```

...takes a GET `/api/auth` endpoint that returns `{enabled: bool, tokens: [{name: string}]}` — names only, never values. That's safe to expose.

**Priya Nair:** Two things that need explicit answering: (1) the `?` about `auth.yaml` vs `config.yaml` — Jordan's "easily viewable" tips this toward a separate `auth.yaml` that can be gitignored cleanly. Mixing secrets into `config.yaml` makes gitignore tricky. Separate file wins. (2) The edge-layer auth question — Jordan didn't mention it, so it's out of scope for v1. orch-tunnel is the transport; auth lives in sdlc.

⚑ **Decided:** Tokens in `auth.yaml` (separate file, `.gitignore`-able). Not in `config.yaml`.

⚑ **Decided:** Edge-layer auth (orch-tunnel server enforcing) is not in v1 scope. sdlc handles its own auth.

---

### Full confirmed scope

All three demands from Session 1 are now designed and confirmed:

**1. orch-tunnel swap** (remove cloudflared)
- `tunnel.rs`: replace `find_cloudflared()` + cloudflared URL extraction with `find_orch_tunnel()` + orch-tunnel stdout pattern
- Error message: "orch-tunnel not found — install with `cargo install orch-tunnel`"
- Named stable URL: `https://sdlc.tunnel.threesix.ai` (predictable, bookmark-friendly)

**2. Auto-start tunnel (default-on)**
- `sdlc ui` starts tunnel by default. No flag needed.
- `sdlc ui --no-tunnel` opts out (local-only)
- If orch-tunnel fails to start: warn to terminal, continue in local-only mode (don't hard-fail)
- UI sidebar: show tunnel status (active URL or "local-only" indicator)

**3. Named token auth (optional, viewable)**
- `.sdlc/auth.yaml` — optional file. Absent = open access (no auth).
- `auth.enabled: false` — explicit disable while keeping tokens listed
- Named tokens, plain strings, hot-reload via mtime watcher
- `GET /api/auth` → `{enabled, tokens: [{name}]}` — no values exposed
- Sidebar panel: token names + enabled status

---

### Milestone structure

One milestone: `v23-network-orch`

Three features (can run in parallel — no shared files except test fixtures):

| Feature slug | Core files |
|---|---|
| `orch-tunnel-swap` | `tunnel.rs`, error messages |
| `tunnel-auto-start` | `cmd/ui.rs`, `ui/Sidebar.tsx` tunnel status |
| `named-token-auth` | `core/auth.rs` (NEW), `server/auth.rs`, `AuthPanel.tsx`, `/api/auth` route |

**Dan Reeves:** These three features have minimal coupling. `orch-tunnel-swap` touches only `tunnel.rs`. `tunnel-auto-start` touches only the CLI flag handling. `named-token-auth` is isolated to the auth layer. They can be specced and implemented in parallel.

---

### What we are NOT building (v1)

- No `sdlc auth add/revoke` CLI (edit YAML directly)
- No bcrypt token storage
- No token expiry
- No audit log / connection tracking
- No edge-layer auth at orch-tunnel server
- No "tunnel unavailable" UI banner (terminal warning is sufficient)

---

### Commit signal status

**MET.** Jordan has confirmed:
- ✓ Auth design agreed (optional, named tokens, easily viewable)
- ✓ Auto-creating (default-on, `--no-tunnel` escape)
- ✓ Overall: "all looks good"

**Next:** `/sdlc-ponder-commit i-need-to-improve-the-way-we-use-network`
