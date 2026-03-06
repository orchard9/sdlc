---
session: 1
timestamp: 2026-03-05T06:15:00Z
orientation:
  current: "Architecture shaped — three-phase migration from token auth to a2api-backed identity, with A2A protocol compliance as Phase 3. Key decisions locked. Phase 1 is ready to build."
  next: "Confirm a2api exposes a token introspection endpoint (or if JWT decode is the only path). Then build Phase 1: token attribution in auth.rs."
  commit: "Phase 1 (token attribution) is small enough to commit immediately as a standalone feature. Phases 2-3 commit when a2api introspection endpoint is confirmed and a second cluster user exists."
---

**jordan · Owner**
A2A Auth & interactions

for the SDLC cluster, we will want to be able to log in using A2A sessions

Preloaded files: desktop-apps.md, oauth.md, README.md

---

**Facilitator**

Two things called "A2A" are in play here and they serve different purposes:

1. **a2api** (orchard9's AI proxy gateway) — has OAuth 2.0 auth code flow, CLI credentials (~/.a2api), refresh token rotation with family revocation. This is the *identity provider*.

2. **Google's A2A protocol** (Agent2Agent) — open interop standard for agent-to-agent communication. Agent Cards declare security schemes (OAuth 2.0, API keys, mTLS, OpenID Connect). JSON-RPC over HTTPS. This is the *interoperability layer*.

The brief says "log in using A2A sessions" — that's about using a2api as the auth backend for sdlc-server's cluster deployment. The A2A protocol question (should sdlc-server publish an Agent Card?) is related but secondary.

Let me bring in two perspectives that will push on different axes of this.

---

**Marcus Webb · Enterprise Platform Architect**

The current token model is fine for today. Named tokens in auth.yaml, hot-reloaded, localhost bypass. For a team of 1-5 accessing a single sdlc-server via Cloudflare tunnel, this works. It's simple, auditable (you can see who has which token by name), and has zero external dependencies.

The moment you need identity-based auth is the moment you need to answer one of:
- "Which human approved this artifact?" → audit attribution
- "Can Jordan's token do things the CI bot's token cannot?" → RBAC
- "Did someone share the QR link with unauthorized people?" → revocation granularity

None of those block you today. But audit attribution WILL be the procurement question. A Fortune 500 CISO won't accept "named tokens and trust."

**My recommendation: plan for a2api OAuth as the cluster auth backend, but don't build it until you have a second team using the cluster deployment.** The migration path matters more than the destination.

On A2A protocol compliance — don't make this a priority. Agent Cards solve a discovery problem: "I'm an agent, I found your Agent Card, now I know how to authenticate." That's for cross-org interop. Your agents already know the URL and auth mechanism because you wrote the skills. Expose `/.well-known/agent.json` as a static file declaring Bearer token auth — it's spec-compliant, costs nothing, gives optionality. But don't refactor auth middleware around A2A abstractions.

On RBAC — resist it. The state machine IS the governance layer. Adding "Jordan can approve but CI bot cannot" creates a second governance layer, and the interaction between them produces invisible bugs. Instead, encode actor-class constraints in the classifier: certain actions require a specific identity class (human vs agent).

On multi-tenancy — single-tenant by design. Each project gets its own sdlc-server instance. Multi-tenancy is fleet orchestration (N instances), not one instance with N tenants. This is the correct architecture for the security story: tenant isolation = process isolation.

---

**Leila Hassan · Platform Engineer**

Three things I want to nail down that Marcus glossed over.

**Token validation: call a2api, do not verify JWTs locally.** a2api uses token family rotation with revocation. Local JWT verification means you can't know a token has been revoked until it expires — you'd need a revocation list, which means rebuilding half of a2api inside sdlc-server. The a2api service is in the same k3s cluster. The network hop is `http://a2api.{namespace}.svc.cluster.local` — sub-millisecond, no TLS, no internet. A 2ms introspection call is operationally free compared to JWKS caching, key rotation, and revocation awareness.

**Internal vs external auth: same middleware, different token sources.** Don't create separate middleware chains.

- External (tunnel): user logged in through a2api's UI, browser has access_token
- Internal (cluster): service accounts use named tokens from auth.yaml, never touch a2api
- Localhost: bypasses everything (sacred, never changes)

One middleware, two validation backends, tried in order.

**The additive pattern is clean.** `is_valid_token` is the single validation point today. Replace with:

```rust
async fn validate_token(config: &TunnelConfig, a2api: Option<&A2apiClient>, token: &str) -> bool {
    if config.is_valid_token(token) { return true; }
    if let Some(client) = a2api { return client.validate(token).await; }
    false
}
```

`A2apiClient` is `Option` in AppState. If `A2API_URL` not set → `None` → identical to today. Same pattern as `DATABASE_URL` for Postgres vs redb. The codebase already has this convention.

For Helm: ExternalSecret for a2api credentials (same pattern as postgres), env vars with `optional: true` so existing deployments keep working.

? Open: does a2api actually expose a token introspection endpoint? The preloaded docs show `/auth/token` for refresh but not an explicit introspection/validation endpoint. Need to confirm before Phase 2.

---

**Facilitator · Synthesis**

Strong convergence. Both perspectives agree on:

⚑ Decided: Three-phase migration
1. **Phase 1 (now):** Token attribution — propagate token name through requests, log identity, git commit trailers. Hours of work, zero new dependencies.
2. **Phase 2 (second cluster user):** a2api as secondary auth backend. `A2API_URL` env var pattern. Introspection calls with TTL cache. Named tokens as permanent fallback.
3. **Phase 3 (enterprise):** `/.well-known/agent.json` for A2A compliance. Actor-class constraints in classifier. OAuth for humans, API keys for machines.

⚑ Decided: sdlc-server is a resource server, never an authorization server. No OAuth redirect flows in sdlc-server.

⚑ Decided: No RBAC layer. State machine + actor-class constraints = governance.

⚑ Decided: No multi-tenancy. Fleet = N instances.

⚑ Decided: `A2API_URL` env var convention (matches `DATABASE_URL`).

⚑ Decided: Localhost bypass is sacred. Never require auth infrastructure for local dev.

? Open: a2api token introspection endpoint — does it exist? Needed before Phase 2 can be specced.

? Open: What identity claims does a2api return on introspection? (email, user_id, roles?)

? Open: Should Phase 1 token attribution flow into the existing `sdlc artifact approve` audit trail, or is git author enough?

---

**Captured artifacts:**
- `architecture-decision.md` — three-phase plan, never-build list, open questions
- `implementation-map.md` — file-by-file change plan for all three phases
