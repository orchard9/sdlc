## Agent Endpoint Config Schema (v1)

### config.yaml additions
```yaml
agent:
  endpoints:
    - id: opus-primary
      provider: anthropic        # restricted to 'anthropic' in v1
      model: claude-opus-4-6
      cooldown_minutes: 5        # optional, default 5 for rate_limit
    - id: sonnet-main
      provider: anthropic
      model: claude-sonnet-4-6
    - id: haiku-cheap
      provider: anthropic
      model: claude-haiku-4-5
      cooldown_minutes: 1        # shorter — haiku less likely to rate limit
  default_chain: [opus-primary, sonnet-main, haiku-cheap]
  action_chains:                 # per-action-type overrides
    create_spec: [haiku-cheap, sonnet-main]
    create_design: [haiku-cheap, sonnet-main]
    approve_spec: [sonnet-main, haiku-cheap]
    implement_task: [opus-primary, sonnet-main]
    approve_review: [opus-primary, sonnet-main]
    create_audit: [sonnet-main, haiku-cheap]
    run_qa: [opus-primary, sonnet-main]
```

### Milestone manifest addition
```yaml
# .sdlc/milestones/<slug>/manifest.yaml
agent_chain: [opus-primary, sonnet-main]   # overrides project default_chain
                                            # does NOT include action_chains override
```

### agent-state.yaml (gitignored — runtime state)
```yaml
# .sdlc/agent-state.yaml
endpoints:
  opus-primary:
    last_failure_at: 2026-03-03T04:50:00Z
    failure_reason: rate_limit           # or 'overloaded'
    cooldown_until: 2026-03-03T04:55:00Z
  sonnet-main:
    # no entry = healthy, available
```

### Resolution priority (highest to lowest)
1. config.agent.action_chains[action_type]  — per-action-type
2. milestone.agent_chain                    — per-milestone
3. config.agent.default_chain              — project default
4. hardcoded 'claude-sonnet-4-6'           — zero-config fallback

### Rust structs (sdlc-core)
```rust
pub struct AgentEndpoint {
    pub id: String,
    pub provider: String,
    pub model: String,
    pub cooldown_minutes: Option<u32>,
}

pub struct AgentConfig {
    pub endpoints: Vec<AgentEndpoint>,
    pub default_chain: Vec<String>,
    pub action_chains: HashMap<String, Vec<String>>,
}

pub struct EndpointState {
    pub last_failure_at: Option<DateTime<Utc>>,
    pub failure_reason: Option<String>,
    pub cooldown_until: Option<DateTime<Utc>>,
}

pub struct AgentState {
    pub endpoints: HashMap<String, EndpointState>,
}
```

### Config struct additions
- `Config.agent: Option<AgentConfig>`
- `Milestone.agent_chain: Option<Vec<String>>`
