---
session: 5
timestamp: 2026-03-03T00:00:00Z
orientation:
  current: "Fleet build complete (v17+v18 released). Auth is the remaining gap — Google OAuth with org restriction, extensible provider trait. 'sdlc is FROZEN' decision is now revised: auth requires targeted Rust changes to sdlc-server."
  next: "Spec the fleet-auth feature — AuthMode enum, Google provider, Helm chart changes, agent token bypass."
  commit: "Auth design is fully shaped and team agrees on provider trait abstraction. Ready to build."
---

## Session 5 — Alignment audit + Auth design

This session covers three things: alignment audit of the ponder workspace against reality,
confirmation that v17+v18 are shipped, and the new auth design that surfaced today.

---

### Alignment Audit

**Misalignments found and corrected:**

1. ⚑ **Manifest was stale.** Sessions 3 and 4 were written directly to `sessions/` bypassing
   `sdlc ponder session log`, so the manifest counter stayed at `2` and status stayed
   `exploring`. Fixed: status updated to `converging`.

2. ⚑ **Architecture open questions are all resolved.** `sdlc-cluster-architecture.md` listed
   5 open questions. All answered by v17/v18 build work. Captured in `build-outcome.md`.

3. ⚑ **"sdlc is FROZEN" decision revised.** Infrastructure remains frozen. Auth requires
   targeted Rust changes to `crates/sdlc-server/src/auth.rs`. Captured in `build-outcome.md`.

4. ⚑ **Auth dimension added.** Google OAuth + org restriction design captured in
   `fleet-auth-design.md`. Absent from all previous sessions.

5. ? **`/sdlc-ponder-commit` was never run.** v17/v18 milestones were created directly. Ponder
   was never formally committed. Leaving as `converging` — commit should happen when auth
   feature is also scoped.

---

### Auth design

**jordan · Owner**

> "its public dns, i think i would just want to build oauth on it for when its deployed in a
> cluster. so if we run it locally its local, if we run it as a cluster we oauth in and then
> thats the name it uses. our team uses google for everything, i want it to be organizational.
> we can build it in a way that we can add other ones in later"

---

**Dan Reeves · Systems Minimalist**

Replace `TunnelConfig` with an `AuthMode` enum — three clean variants: `None`, `Tunnel`,
`OAuth`. Old behavior preserved exactly, new behavior isolated. No if-chains proliferating.

⚑ Decided: `AuthMode` enum replaces `TunnelConfig`. Three variants. Detection at startup from env vars.

---

**Leila Hassan · Platform Engineering Lead**

Google's `hd` param enforces org restriction at the authorization server — not just a UI
hint. Set `hd=yourorg.com` in the auth URL + validate `hd` claim from userinfo on callback.
Stateless HMAC-signed session cookie: 32-byte secret, validate on every request. No session
store dependency.

⚑ Decided: Stateless HMAC-signed session cookie. `hd` param + claim validation for org.

---

**Priya Nair · Distributed Systems Architect**

Agent identity: `SDLC_AGENT_TOKEN` env var injected via Helm. Bearer token header bypasses
OAuth, stamps `agent` in `approved_by`. `approved_by` becomes real — human approvals carry
email, agent approvals carry "agent". Audit trail is finally meaningful.

? Open: What is the orchard9 Google Workspace domain? Needed for `SDLC_AUTH_ALLOWED_DOMAIN`.

---

### ⚑ Decisions

1. Auth is required before fleet goes live on public DNS.
2. `AuthMode` enum replaces `TunnelConfig`. Variants: `None`, `Tunnel`, `OAuth`.
3. Google OAuth with `hd` param + extensible provider trait for future providers.
4. Stateless HMAC-signed session cookie. No Redis, no DB.
5. `SDLC_AGENT_TOKEN` bearer token bypass for agent runs.
6. "sdlc is FROZEN" revised — auth requires targeted Rust changes to sdlc-server.

### ? Open

1. Orchard9 Google Workspace domain? (for `SDLC_AUTH_ALLOWED_DOMAIN`)
2. Commit auth as part of this ponder or separate?

---

## Product Summary

### What we explored
Alignment audit of the fleet ponder workspace against what was actually built (v17+v18),
and design of the auth layer needed before fleet pods can go live on public DNS.

### Key shifts
The "sdlc is FROZEN" architecture decision from session 4 is revised — auth requires
targeted Rust changes. The fleet infrastructure (Helm chart, pipelines) stays frozen, but
sdlc-server needs an AuthMode enum, OAuth routes, and HMAC session cookies. The Google
OAuth design with org restriction and extensible provider trait is now captured and agreed.

### Implications
Fleet cannot go live on public DNS without auth. Auth is the single gate between "all
artifacts validated in isolation" and "fleet is actually running." One scoped feature
(fleet-auth) unblocks the end-to-end deployment. It should be sequenced before bootstrap.

### Still open
1. What is the orchard9 Google Workspace domain? (required to build the Google provider)
2. Should auth be committed as part of this ponder or separately?
