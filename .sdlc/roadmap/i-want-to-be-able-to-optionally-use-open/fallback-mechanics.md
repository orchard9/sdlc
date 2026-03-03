## Fallback Chain Mechanics

### Trigger conditions
- `rate_limit` — Claude API 429 / RPM or TPM limit reached. Cooldown: 5 min (configurable)
- `overloaded` — Claude API overloaded_error. Cooldown: 1 min (configurable)
- Other errors (auth, network, code errors) → DO NOT trigger fallback. Surface immediately.

### Algorithm
```
fn start_feature_run(slug, action_type, resolved_chain, app):
  state = load_agent_state(root)
  for endpoint_id in resolved_chain:
    endpoint = config.find_endpoint(endpoint_id)
    if \!state.is_available(endpoint_id):
      continue  // on cooldown
    result = spawn_agent_run_with_model(slug, endpoint.model, app)
    match result:
      RateLimit | Overloaded(reason) →
        state.mark_cooldown(endpoint_id, reason)
        save_agent_state(root, state)
        emit_sse(AgentEndpointCooldown { endpoint_id, cooldown_until })
        continue  // try next in chain
      Success → return Ok(result)
      OtherError → return Err(result)  // don't fall through
  // exhausted all options
  emit_sse(AgentQuotaExhausted { slug, tried: resolved_chain })
  return Err(QuotaExhausted)
```

### SSE events
- `AgentEndpointFallback { feature_slug, from_endpoint, to_endpoint }`
- `AgentEndpointCooldown { endpoint_id, cooldown_until }`
- `AgentEndpointRecovered { endpoint_id }`  (emitted when cooldown expires + checked)
- `AgentQuotaExhausted { feature_slug, tried: Vec<String> }`

### REST API
- GET  /api/agent/config            → AgentConfig (from config.yaml)
- PATCH /api/agent/config           → update config.yaml
- GET  /api/agent/state             → AgentState (from agent-state.yaml)
- POST /api/agent/state/:id/reset   → clear cooldown for endpoint
- POST /api/agent/state/reset-all   → clear all cooldowns (dev-mode convenience)

### Mid-run failure handling
When an agent run exits mid-way due to rate limit:
- The run record is marked failed with reason 'rate_limit'
- The endpoint is cooled down
- A new run is immediately retried with the next endpoint in chain
- New run prompt includes: 'Previous attempt was interrupted by rate limiting. Start from the beginning.'
- Both runs appear in run history (one failed, one succeeded/pending)

### Cooldown state file
- Location: .sdlc/agent-state.yaml
- Gitignored — this is per-developer transient runtime state
- GC: on each write, prune entries for endpoints not in current config
- Format: YAML, overwritten atomically (via sdlc-core io.rs atomic_write)

### Interaction with daily_budget_usd
- Budget cap (observability.daily_budget_usd) is a global kill-switch
- When daily budget is hit, NO runs start — fallback chain doesn't help
- These two systems are orthogonal: quota fallback handles rate limits, budget cap handles cost limits
