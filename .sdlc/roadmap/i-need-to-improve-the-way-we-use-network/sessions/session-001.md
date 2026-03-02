---
session: 1
timestamp: 2026-03-02T20:45:00Z
orientation:
  current: "Three demands with different urgency levels: (1) swap cloudflared → orch-tunnel [clear, low risk], (2) auto-start tunnel on sdlc ui [simple flag flip], (3) 'code-based auth' [needs definition]. Design session surfaced a strong candidate: orch-tunnel named tunnels give predictable URLs + Jordan controls the edge, so auth can be layered at the edge, at the token level, or both."
  next: "Get Jordan to confirm what 'code-based auth' means — is it: (a) sdlc manages an API key list in .sdlc/, (b) orch-tunnel server enforces auth at the edge, or (c) replace the QR/cookie mechanism with something Jordan can script."
  commit: "Auth design is agreed on. Then all three changes can be implemented in one milestone."
---

**jordan · Owner**
i need to improve the way we use network

i want the tunnels to be auto creating
i want to remove cloudflare as a tunnel
i want to only use the orch-tunnel as a tunnel
i need to put code based auth in it properly so that i control the access that have

---

## Session 1 — Network & Tunnel Architecture

**Facilitator note:** I recruited Priya Nair (distributed systems), Dan Reeves (systems minimalist), and Leila Hassan (platform engineering). Three distinct lenses. Let's work through Jordan's four demands systematically.

---

### Reading the current code first

Before opinions, let's be precise about what exists:

**Current tunnel stack (as-shipped):**
- `crates/sdlc-server/src/tunnel.rs` — spawns `cloudflared tunnel --url http://localhost:{port}`, reads `*.trycloudflare.com` URL from stderr
- `crates/sdlc-cli/src/cmd/ui.rs` — `--tunnel` flag calls `Tunnel::start()` → cloudflared
- Auth: random 8-char token, passed via `?auth=TOKEN` query param, stored as `sdlc_auth` cookie; QR code embeds the auth URL
- `docs/orch-tunnel-reference.md` — `orch-tunnel` already exists and is documented. CLI: `orch-tunnel http <port> --name <slug>`. Named tunnels produce `https://{name}.tunnel.threesix.ai`. Jordan controls the edge server at `connect.tunnel.threesix.ai:7844`.

The orch-tunnel is already designed for this. It's not a new thing to build. It's a swap.

---

### Demand 1: Remove cloudflare, use only orch-tunnel

**Priya Nair:** This is the right call. cloudflared is a third-party dependency Jordan doesn't control. The URL is random (`fancy-rabbit-xyz.trycloudflare.com`) — it changes every restart, which breaks bookmark-based access. orch-tunnel gives a *named, stable* URL: `https://sdlc.tunnel.threesix.ai`. That's the project name. It won't change between restarts.

**Dan Reeves:** How much code changes? The swap is straightforward:
1. Delete `find_cloudflared()` and the cloudflared-specific URL extraction (`extract_tunnel_url` — looks for `.trycloudflare.com`)
2. Replace with `find_orch_tunnel()` (uses `which::which("orch-tunnel")`)
3. Replace the stderr URL extraction — orch-tunnel outputs to stdout, URL format is `https://{name}.tunnel.threesix.ai`, predictable
4. `TunnelError::NotFound` becomes "orch-tunnel not found" with new install instructions

That's `tunnel.rs` rewrite + error messages. Auth middleware (`auth.rs`) and the route handlers (`routes/tunnel.rs`) are untouched. Net change: ~100 lines.

⚑ **Decided:** Replace cloudflared with orch-tunnel. Tunnel binary detection, URL extraction, and error messages all update. The auth layer stays as-is for now (separate concern).

---

### Demand 2: Auto-creating tunnels

**Jordan:** "auto creating" — but what does this actually mean?

**Dan Reeves:** Two interpretations:
- **A) Default on:** `sdlc ui` starts a tunnel by default; `--no-tunnel` to opt out
- **B) Always on:** no flag at all; tunnel always starts; no opt-out

I'd push back on (B). There are valid scenarios where Jordan wants local-only access: air-gapped machines, development, or when the tunnel edge is unreachable. Default-on (A) with `--no-tunnel` escape hatch is the minimum change that gives Jordan what he wants without trapping future users.

**Priya Nair:** Named tunnel has a nice side-effect here: `https://sdlc.tunnel.threesix.ai` is stable. If sdlc restarts, the same URL comes back (same `--name sdlc`). So auto-start doesn't break existing browser sessions the way cloudflared did. This makes default-on much more appealing.

**Leila Hassan:** If orch-tunnel fails to start (network down, edge unreachable), the current code returns an error and halts. With auto-start as default, that becomes a forced failure for every `sdlc ui` in offline environments. The graceful behavior should be: log a warning, continue in local-only mode. Don't fail hard.

⚑ **Decided:** Make `--tunnel` the default (no flag needed). Add `--no-tunnel` to opt out. If orch-tunnel fails to start, log a warning and continue in local-only mode — don't hard-fail.

**? Open:** Should the UI show "tunnel unavailable" in the sidebar when local-only mode triggers? Or is a terminal warning sufficient?

---

### Demand 3: Code-based auth — the ambiguous one

This is the design question. Jordan says "I control the access." What does that mean in practice?

The current auth mechanism:
- `generate_token()` produces a random 8-char alphanumeric token on startup
- Token is embedded in the QR code URL as `?auth=TOKEN`
- First visit with that URL sets an `sdlc_auth` cookie
- Token is not persisted — it changes on every restart

**Problems with current auth:**
1. Token rotates on restart — anyone who had access loses it
2. Jordan can't grant or revoke access to a specific person without restarting the whole server
3. There's no audit trail — no way to know who accessed the UI
4. Can't script access — can't pass a token to an agent or CI system without QR scanning

**Three candidate designs:**

**Option A: Named tokens in `.sdlc/config.yaml` (or `.sdlc/auth.yaml`)**
```yaml
# .sdlc/auth.yaml
tokens:
  - name: jordan-phone
    token: "abc123"
  - name: ci-agent
    token: "xyz987"
  - name: guest-review
    token: "def456"
    expires: "2026-03-10"
```
Jordan manages the file. sdlc server reads it on startup (and watches for changes). Each token is named so Jordan can revoke by name. `?auth=jordan-phone-token-value` sets a persistent cookie.

**Dan Reeves:** I like this. It's YAML. It's already the storage model. No new concepts. Jordan can git-commit the token list (sans actual token values if he uses `.gitignore`).

**Priya Nair:** Token rotation problem: if Jordan adds a token while the server is running, does it take effect without restart? File watcher on `auth.yaml` with hot-reload would handle this. The mtime watcher infrastructure already exists in `state.rs`.

**Option B: Edge-layer auth (orch-tunnel server enforces)**
Since Jordan controls `connect.tunnel.threesix.ai`, he could enforce auth at the edge — the tunnel server only forwards requests with a valid header. sdlc server itself stays auth-free (local-bypass mode always).

**Leila Hassan:** Clean separation. The tunnel server is the security boundary. sdlc doesn't do auth at all. But this means auth config is on the tunnel server, not in the project. If someone bypasses the tunnel and hits the local port, there's no auth.

**Dan Reeves:** The local port should still have no auth (that's the current behavior — `localhost` bypasses auth). This is actually fine. The tunnel is the only public exposure. If you're on localhost, you're on the machine.

**? Open:** Does Jordan want sdlc-layer auth in addition to tunnel-layer auth, or is tunnel-layer auth sufficient?

**Option C: Token file + API key management CLI**
`sdlc auth add <name>`, `sdlc auth revoke <name>`, `sdlc auth list` — manages `.sdlc/auth.yaml`. Tokens are stored as bcrypt hashes (never plaintext). Jordan gets a CLI to manage access without touching YAML directly.

**Tobias Krenn** (via Dan relaying Tobias's skepticism): This is three features disguised as one. Start with Option A (YAML file, plain tokens) since this is a single-user tool. If the token management overhead becomes real friction, then build the CLI. Don't pre-build admin UX for a problem that doesn't exist yet.

---

### Synthesizing the auth direction

The simplest design that gives Jordan control:

**Proposal: Persistent named tokens in `.sdlc/auth.yaml`**

1. `auth.yaml` lists named tokens. Jordan edits it directly.
2. sdlc server reads `auth.yaml` on startup; hot-reloads when it changes (mtime watcher, same pattern as feature files).
3. Any valid token grants access. Token is stored as the cookie value.
4. If `auth.yaml` doesn't exist, generate a single random token (backward-compatible behavior) and print it to terminal.
5. A `--no-auth` flag (or `auth: disabled` in config) keeps the current behavior for local-only setups.

What Jordan gets:
- `auth.yaml` is the roster. He can share tokens with agents, CI, specific people.
- Revoking: delete or rename the token in `auth.yaml`. Server picks it up without restart.
- Stable tokens: token doesn't rotate on restart.
- Named tokens: he knows which session is which.

⚑ **Decided:** Auth design is Option A — named tokens in `.sdlc/auth.yaml` (or `tokens:` section in `.sdlc/config.yaml`). No bcrypt in v1 — plain tokens are fine for a personal tool. Hot-reload via mtime watcher.

**? Open:** Should tokens live in `auth.yaml` (separate file, easy to gitignore) or in `config.yaml` (single file, but harder to gitignore secrets)?

---

### Implementation scope summary

**What changes:**

| File | Change |
|------|--------|
| `crates/sdlc-server/src/tunnel.rs` | Replace cloudflared with orch-tunnel. New `find_orch_tunnel()`, new URL extraction (stdout, `.tunnel.threesix.ai` pattern), new error messages. |
| `crates/sdlc-server/src/auth.rs` | Replace single-token `TunnelConfig` with multi-token `AuthConfig` that reads from `auth.yaml`. Hot-reload via mtime watcher. |
| `crates/sdlc-cli/src/cmd/ui.rs` | Flip `--tunnel` from opt-in to opt-out (`--no-tunnel`). Graceful degradation if orch-tunnel fails. |
| `crates/sdlc-core/src/auth.rs` (new) | `AuthConfig` struct, `auth.yaml` load/save, token validation. |
| `docs/orch-tunnel-reference.md` | Update to reflect new default behavior. |

**What doesn't change:**
- Cookie mechanism (`sdlc_auth` cookie) — already correct
- `?auth=TOKEN` bootstrap flow — already correct
- Local bypass (`localhost` always passes) — keep
- `/__sdlc/` always-public rule — keep

---

### Risks and tensions

**Priya Nair:** The graceful-degradation path (orch-tunnel fails → local-only) needs to be clearly communicated in the UI, not just the terminal. If an agent is trying to use the tunnel URL and the tunnel failed silently, the agent will hit connection errors with no explanation.

**Dan Reeves:** Don't build the token management CLI in v1. Ship the YAML file. If Jordan finds himself editing YAML too often, that's the signal. Not before.

**Leila Hassan:** orch-tunnel's edge server uptime becomes a dependency. Currently cloudflared has near-100% uptime (Cloudflare runs it). Jordan's orch-tunnel server could go down. The graceful degradation needs to be reliable, not just coded.

---

### Next move

Before committing to a milestone:

1. **Confirm auth design with Jordan** — specifically: (a) named tokens in separate `auth.yaml` vs. in `config.yaml`, and (b) whether edge-layer auth at orch-tunnel server is in scope or a separate concern
2. **Confirm "auto-creating" scope** — default-on with `--no-tunnel` escape, or always-on with no escape?
3. Once confirmed → one milestone: `v-network-tunnel-orch` with three features: (1) orch-tunnel swap, (2) default tunnel, (3) named token auth
