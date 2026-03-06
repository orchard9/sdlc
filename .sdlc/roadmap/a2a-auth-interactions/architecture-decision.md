# Architecture Decision: Auth Migration Path

## ⚑ Decided: Three-Phase Migration

### Phase 1 — Token Attribution (now, hours of work)
- Propagate token name through request lifecycle after `is_valid_token` matches
- Inject identity (token name) into request extensions
- Log identity on every API call
- Include identity in git commit trailers when server makes commits
- **No new dependencies, no external services**

### Phase 2 — a2api as Secondary Auth (when second cluster user exists)
- Add `A2apiClient` to AppState (`Option`, constructed from `A2API_URL` env var)
- Bearer token check: try auth.yaml first, then a2api introspection
- a2api validation via internal cluster URL: `http://a2api.{namespace}.svc.cluster.local`
- Cache validated tokens for ~55 minutes (tokens live 1 hour)
- ExternalSecret for OAuth client credentials (same pattern as postgres)
- **sdlc-server is a resource server, NOT an authorization server** — it validates tokens, never runs OAuth redirect flows

### Phase 3 — a2api as Primary (enterprise)
- Static `/.well-known/agent.json` declaring auth scheme (A2A protocol compliance)
- Named tokens become 'service accounts' for CI/agents
- OAuth for humans, API keys for machines (Stripe pattern)
- Identity-class constraints in classifier rules (human vs agent actor types)

## ⚑ Decided: Never Build
- Multi-tenant sdlc-server (fleet orchestration = N instances, not 1 with N tenants)
- RBAC as separate layer from state machine (state machine IS the governance)
- Database-backed session storage
- Full OAuth authorization code flow in sdlc-server
- Network policies (premature for current cluster size)

## ? Open: Token Introspection vs Local JWT
- Leila argues: call a2api for validation (sub-ms in-cluster, handles revocation)
- Marcus argues: same direction but notes JWT decode could work with JWKS caching
- Consensus leans toward introspection — simpler, handles token family revocation automatically
- Need to confirm: does a2api expose a token introspection endpoint?