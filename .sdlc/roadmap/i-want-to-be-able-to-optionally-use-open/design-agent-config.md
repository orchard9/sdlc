## Design: Agent Config Per Milestone

### Two-level config hierarchy

```yaml
# .sdlc/config.yaml  (project default)
agent:
  model: claude-sonnet-4-6

# .sdlc/milestones/<slug>/manifest.yaml  (milestone override)
agent:
  model: claude-opus-4-6
  executor: claude   # reserved, not implemented until spike
```

### Lookup chain in spawn_agent_run
1. Find which milestone contains the feature slug (scan milestones dir)
2. Read `milestone.agent.model` if set
3. Fall back to `config.agent.model` if set
4. Fall back to hardcoded `claude-sonnet-4-6`

### Data model additions
- `AgentConfig { model: Option<String>, executor: Option<String> }`
- `Milestone.agent: Option<AgentConfig>`
- `Config.agent: Option<AgentConfig>`

### New function needed
```rust
fn milestone_for_feature(root: &Path, feature_slug: &str) -> Option<Milestone>
```
Scans .sdlc/milestones/*/manifest.yaml for a features list containing the slug.

### OpenCode question (OPEN)
OpenCode (opencode.ai) is a TUI — does it have a programmatic non-interactive mode?
If not, 'use OpenCode executor' cannot be wired into spawn_agent_run without an adapter.
Spike needed before executor switching is implementable.