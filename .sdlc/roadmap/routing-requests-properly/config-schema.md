## Proposed Config Schema — routing in .sdlc/config.yaml

```yaml
# .sdlc/config.yaml addition
routing:
  defaults:
    provider: claude_code          # claude_code | opencode | gemini_cli
    model: claude-sonnet-4-6       # any model string passed as --model
  
  skills:
    # High-stakes autonomous runs — upgrade to opus
    feature_run:
      provider: claude_code
      model: claude-opus-4-6
    
    milestone_run_wave:
      provider: claude_code
      model: claude-opus-4-6
    
    # Creative/ideation — could use gemini later
    ponder_chat:
      provider: claude_code
      model: claude-sonnet-4-6
    
    investigation_chat:
      provider: claude_code
      model: claude-sonnet-4-6
    
    # Lightweight tasks — haiku works fine
    ama_answer:
      provider: claude_code
      model: claude-haiku-4-5
    
    # Default omitted = use routing.defaults
```

**Data layer change (sdlc-core/src/config.rs):**
```rust
#[derive(Debug, Deserialize, Default)]
pub struct RoutingConfig {
    pub defaults: ProviderConfig,
    pub skills: HashMap<String, ProviderConfig>,
}

#[derive(Debug, Deserialize)]
pub struct ProviderConfig {
    pub provider: Option<Provider>,
    pub model: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
pub enum Provider {
    #[default]
    ClaudeCode,
    OpenCode,
    GeminiCli,
}
```

**Server-side change (sdlc_query_options → routing-aware):**
```rust
pub(crate) fn sdlc_query_options(
    root: std::path::PathBuf,
    max_turns: u32,
    skill: &str,          // ← new param
    config: &SdlcConfig,  // ← read routing table
) -> QueryOptions {
    let routing = config.routing_for(skill);
    QueryOptions {
        model: routing.model.clone(),
        path_to_executable: routing.executable_path(), // 'claude', 'opencode', 'gemini'
        ..standard_options
    }
}
```

**⚑ Decided: Config lives in .sdlc/config.yaml** — consistent with all other sdlc config, per-project routing makes sense (different projects may want different models).