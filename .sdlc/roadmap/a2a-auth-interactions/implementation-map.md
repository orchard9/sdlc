# Implementation Map

## Files to Modify

### Phase 1 — Token Attribution
| File | Change |
|------|--------|
| `crates/sdlc-server/src/auth.rs` | After `is_valid_token` succeeds (lines ~123, ~139), extract token name from `tokens` vec, inject into request extensions |
| `crates/sdlc-server/src/state.rs` | Add identity type to request extensions |

### Phase 2 — a2api Validation
| File | Change |
|------|--------|
| `crates/sdlc-server/src/auth.rs` | Add `validate_token()` async fn: try auth.yaml, then a2api |
| `crates/sdlc-server/src/state.rs` | Add `Option<A2apiClient>` to AppState, construct from `A2API_URL` env |
| `crates/sdlc-server/src/a2api_client.rs` | New file: `A2apiClient` struct with `reqwest::Client`, TTL cache, `validate()` method |
| `k3s-fleet/deployments/helm/sdlc-server/templates/deployment.yaml` | Add `A2API_URL`, `A2API_CLIENT_ID`, `A2API_CLIENT_SECRET` env vars (optional: true) |
| `k3s-fleet/deployments/helm/sdlc-server/templates/external-secret-a2api.yaml` | New: ExternalSecret for a2api credentials via gcp-secret-manager |
| `k3s-fleet/deployments/helm/sdlc-server/values.yaml` | Add a2api section |

### Phase 3 — A2A Protocol Compliance
| File | Change |
|------|--------|
| `crates/sdlc-server/src/routes/` | New: `/.well-known/agent.json` static endpoint |
| `crates/sdlc-core/src/rules.rs` | Identity-class constraints on actions |
| `crates/sdlc-core/src/types.rs` | `ActorType` enum (human, agent, service) |

## Env Var Convention (matches DATABASE_URL pattern)
- `A2API_URL` set → a2api auth active
- `A2API_URL` not set → existing behavior (auth.yaml tokens only)
- Named tokens ALWAYS work as fallback (even when a2api is configured)