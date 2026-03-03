---
session: 1
timestamp: 2026-03-03T03:30:00Z
orientation:
  current: "Route inventory complete. Config schema designed. Phase split proposed: Phase 1 = model routing for Claude Code only. Phase 2 = provider routing (OpenCode/Gemini CLI) pending protocol spike."
  next: "Run a technical spike on OpenCode CLI stream-json protocol compatibility, then commit to milestone if compatible or define Phase 2 scope separately."
  commit: "Ready when (1) spike confirms or denies OpenCode stream-json compatibility, and (2) jordan confirms Phase 1 (model-only routing) is sufficient to ship first."
---

**jordan · Owner**
> routing requests properly
>
> our server -> MCP integration needs to be able to route traffic properly, we should be able to use open code for tasks that we want to use gemini models and pick the proper model, and claude code for tasks we want the sonnet and opus and that agentive workflow dynamically
>
> this means we have to list all of the routes that call models and then be able to dynamically configure them
>
> the configuration should be (1) skill, (2) agentive provider (claude code or open code)
> (3) model (sonnet-4.6, opus-4.6, etc)

---

## Session 1 — Route Inventory + Design

### Orienting on the codebase

Before designing anything, I did a full read of the current architecture:

- `crates/claude-agent/src/process.rs` — spawns `claude --output-format stream-json --input-format stream-json`
- `crates/sdlc-server/src/routes/runs.rs` — `sdlc_query_options()` is the **single factory** that all 15+ agent routes use
- `crates/claude-agent/src/types.rs` — `QueryOptions` has two relevant fields: `model: Option<String>` and `path_to_executable: Option<String>`

The chokepoint is clear: **`sdlc_query_options(root, max_turns)` — everything flows through here.**

---

### Route Inventory

I catalogued all `spawn_agent_run` callsites and proposed canonical skill names:

| Skill | Handler | Notes |
|---|---|---|
| `feature_run` | `start_run` | Drives a feature, 200 turns, heavy |
| `milestone_uat` | `start_milestone_uat` | Playwright MCP, needs trust |
| `milestone_prepare` | `start_milestone_prepare` | 100 turns, alignment work |
| `milestone_run_wave` | `start_milestone_run_wave` | 200 turns, parallel features |
| `ponder_chat` | `start_ponder_chat` | 100 turns, creative/ideation |
| `ponder_commit` | `commit_ponder` | Crystallize ponder to milestones |
| `investigation_chat` | `start_investigation_chat` | Root-cause / evolve / guideline |
| `vision_align` | `start_vision_align` | 40 turns, doc alignment |
| `architecture_align` | `start_architecture_align` | 40 turns, code alignment |
| `team_recruit` | `start_team_recruit` | 40 turns, agent creation |
| `ama_answer` | `answer_ama` | 5 turns, lightweight Q&A |
| `quality_reconfigure` | `reconfigure_quality_gates` | 10 turns |
| `quality_fix` | `fix_quality_issues` | 20 turns |
| `tool_plan` | `plan_tool` | 15 turns |
| `tool_build` | `build_tool` | 25 turns |
| `tool_evolve` | `evolve_tool` | 20 turns |
| `tool_act` | `act_tool` | 20 turns |

---

### The OpenCode Compatibility Problem

**This is the session's biggest tension.**

Jordan wants to use OpenCode to run Gemini models. But `claude-agent` crate speaks a very specific protocol: the Claude CLI's `--output-format stream-json --input-format stream-json` bidirectional JSONL interface.

**The question: does OpenCode speak the same protocol?**

OpenCode (by SST) is a TUI coding assistant. From its architecture, it is **not** a subprocess-streaming tool — it's interactive terminal UI. It does not expose `--output-format stream-json` flags. It would require a **new process driver** in the `claude-agent` crate to be a viable provider.

**?  Open: Does `opencode` CLI support stream-json subprocess mode?**
This is a required spike before committing to "opencode" as a provider type.

**Priya Nair (Distributed Systems perspective) surfaces a constraint:**
"The stream-json protocol is what gives you structured event delivery, tool call/result pairing, and run record persistence. If you fork the driver for OpenCode, you also have to handle its message schema differences in `stream.rs`, `runner.rs`, and potentially the SSE emission layer. That's not a small lift. Get the protocol answer first — don't design the abstraction until you know if it collapses to one protocol or needs two."

**Alternative provider: Gemini CLI (`gemini`)**
Gemini CLI is a subprocess agent that does support `--output-format stream-json` in a compatible mode. If the goal is "use Gemini models," routing through Gemini CLI is more viable than routing through OpenCode.

⚑ **Decided: We need a spike on OpenCode stream-json compatibility before adding it as a provider type.** Phase 1 can ship without it.

---

### Phase Split

**Phase 1 — Model Routing (ship-ready now):**
- Add `routing` section to `.sdlc/config.yaml`
- Map skill names → `{ model: "claude-opus-4-6" }` for Claude Code
- `sdlc_query_options` gets a `skill: &str` param, reads routing config, sets `opts.model`
- Zero protocol changes, zero new drivers
- Immediate value: run `feature_run` on opus, `ama_answer` on haiku

**Phase 2 — Provider Routing (needs spike first):**
- If OpenCode is subprocess-compatible → add `provider: opencode` support, probably just `path_to_executable = "opencode"` with model passthrough
- If OpenCode is NOT subprocess-compatible → add Gemini CLI as provider (`provider: gemini_cli`), possibly with a thin adapter
- Full multi-provider dispatch table

---

### Config Schema (Phase 1)

```yaml
# .sdlc/config.yaml addition
routing:
  defaults:
    provider: claude_code
    model: claude-sonnet-4-6
  skills:
    feature_run:
      provider: claude_code
      model: claude-opus-4-6
    milestone_run_wave:
      provider: claude_code
      model: claude-opus-4-6
    ama_answer:
      provider: claude_code
      model: claude-haiku-4-5
    # others inherit defaults
```

**Rust data shape (sdlc-core/src/config.rs):**
```rust
#[derive(Debug, Deserialize, Default)]
pub struct RoutingConfig {
    pub defaults: ProviderConfig,
    pub skills: HashMap<String, ProviderConfig>,
}

#[derive(Debug, Deserialize, Default)]
pub struct ProviderConfig {
    pub provider: Option<Provider>,
    pub model: Option<String>,
}

#[derive(Debug, Deserialize, Default, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Provider {
    #[default]
    ClaudeCode,
    OpenCode,
    GeminiCli,
}
```

**sdlc-server change:**
```rust
// sdlc_query_options gains skill param + config lookup
pub(crate) fn sdlc_query_options(
    root: std::path::PathBuf,
    max_turns: u32,
    skill: &str,
    routing: &RoutingConfig,
) -> QueryOptions {
    let route = routing.skills.get(skill)
        .unwrap_or(&routing.defaults);
    let model = route.model.clone()
        .or_else(|| routing.defaults.model.clone());
    let executable = match route.provider.as_ref()
        .or(routing.defaults.provider.as_ref()) {
        Some(Provider::OpenCode) => Some("opencode".to_string()),
        Some(Provider::GeminiCli) => Some("gemini".to_string()),
        _ => None, // default: "claude"
    };
    QueryOptions {
        model,
        path_to_executable: executable,
        ..standard_options(root, max_turns)
    }
}
```

All 17 callsites change from `sdlc_query_options(app.root.clone(), N)` to `sdlc_query_options(app.root.clone(), N, "skill_name", &config.routing)`.

---

### UI Implication

**?  Open: Should the frontend show which provider/model a run is using?**
The `RunRecord` in `state.rs` could carry `provider` and `model` fields, making them visible in the run cards. This would help jordan see at a glance which runs used opus vs. sonnet.

**Ben Hartley (Dev UX) perspective:**
"Show it, but make it secondary. The skill and status are primary. Provider/model is metadata — maybe a small tag on the run card, not the lead. And definitely show it in the run detail panel since that's where debugging happens."

---

### Implementation Scope for a Feature

A feature `provider-model-routing` would need:

**sdlc-core:**
- Add `RoutingConfig`, `ProviderConfig`, `Provider` to `config.rs`
- Add `routing: RoutingConfig` field to `SdlcConfig`
- Add `fn routing_for(&self, skill: &str) -> &ProviderConfig` helper

**sdlc-server/routes/runs.rs:**
- Change `sdlc_query_options` signature to accept `skill: &str` and `routing: &RoutingConfig`
- Update all 17 callsites
- Pass `app.config.routing` into all calls
- Add `provider` and `model` fields to `RunRecord`

**sdlc-cli (optional):**
- `sdlc routing show` — list the current routing table
- `sdlc routing set <skill> --model <model> --provider <provider>`

**Frontend:**
- Show provider/model tag on `RunCard`
- (Phase 2) Routing config panel in settings UI

---

### Open Tensions Summary

**?  Open: OpenCode stream-json protocol compatibility** — needs spike before Phase 2 design

**?  Open: Should `routing` config be project-level (.sdlc/config.yaml) or user-level (~/.sdlc/config)?**
Currently leaning project-level because different projects might want different model policies. But a user-level override would let you set a global "always use opus for feature_run" without touching each project. Could support both (user-level default, project overrides).

**?  Open: Hot-reload vs restart?** The server currently reads config at startup. Should routing config changes require a restart? Hot-reload is possible (config re-read per request) but adds complexity. For now: restart is fine.

⚑ **Decided: Phase 1 = model routing only, Claude Code only.** Ships clean, immediate value.

⚑ **Decided: Canonical skill name taxonomy is the 17 names in the route inventory table.**

⚑ **Decided: Config lives in .sdlc/config.yaml under a `routing:` key.**

---

## What's Ready to Build

**Phase 1 feature** (`provider-model-routing`) is fully designed and ready to execute:
1. Data types in sdlc-core/config.rs
2. `sdlc_query_options` signature change + 17 callsite updates in runs.rs
3. RunRecord carries model/provider fields
4. Frontend RunCard shows model tag

**Spike needed first** for Phase 2 (OpenCode provider): run `opencode --help` and test if `--output-format stream-json` flag exists, then confirm JSONL message schema matches claude-agent types.
