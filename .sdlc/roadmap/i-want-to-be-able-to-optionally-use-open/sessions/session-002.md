---
session: 2
timestamp: 2026-03-03T05:00:00Z
orientation:
  current: "Scope expanded: full endpoint config system (named endpoints, multi-level fallback chains, per-action-type routing) + left-panel config screen. Session 1's two-level hierarchy still valid but becomes endpoints-first. 'Skill' = ActionType (confirmed by design). Quota = stateful error-based detection in agent-state.yaml (gitignored). Config screen = dedicated Endpoints page in sidebar."
  next: "Write the feature spec for `agent-endpoint-config`. Three sub-features: (1) data model: endpoints + chains in config.yaml + agent-state.yaml, (2) server-side fallback retry logic in spawn_agent_run, (3) frontend Endpoints page. Per-action-type routing via directive pre-read at spawn time — clean and cheap."
  commit: "Scope settled: endpoints + chains + config screen is v1. Per-action-type routing ships in v1 (it's cheap: classify before spawn). Cross-provider fallback is later. agent-state.yaml is gitignored transient runtime state."
---

**jordan · Owner**
really pressure test this and make sure we handle a dynamic set of cases - we can build a full config screen in the left that lets users control details for each endpoint/skill

we can also let them set up preference fallback - if x quota, then y quota, then z quota

---

## Session 2 — Pressure Testing: Endpoints, Fallback Chains, Config Screen

### Opening reorientation

Session 1 landed on a focused v1: `agent.model` in milestone manifest + project config, two-level lookup, no UI. That was the right minimum.

Jordan has now seeded with something significantly richer: a **full config screen**, **per-endpoint/skill control**, and **ordered fallback chains triggered by quota exhaustion**.

This isn't an incremental addition — it's a design-phase upgrade that warrants rebuilding the mental model before locking scope. I'm pulling in **Dan Reeves**, **Tobias Krenn**, **Ben Hartley**, and **Dana Cho** for this one.

Three new concepts to interrogate:
1. **"Endpoint"** — named configuration unit (provider + model + key reference)?
2. **"Skill"** — action type (create_spec, implement_task)? feature category? sdlc slash command?
3. **"If x quota, then y quota, then z quota"** — static user declaration or dynamic error detection?

---

### Unpacking "endpoint"

An "endpoint" in this context means:

- A **named, reusable configuration** for one model call destination
- Contains: `id`, `provider`, `model`
- The same endpoint definition is referenced by multiple chains

This is meaningfully different from "just put a model string in the manifest." An endpoint is a named resource shared across milestone configs.

Proposed minimal endpoint definition:
```yaml
# .sdlc/config.yaml
agent:
  endpoints:
    - id: opus-primary
      provider: anthropic
      model: claude-opus-4-6
    - id: sonnet-main
      provider: anthropic
      model: claude-sonnet-4-6
    - id: haiku-cheap
      provider: anthropic
      model: claude-haiku-4-5
  default_chain: [opus-primary, sonnet-main, haiku-cheap]
```

---

### Dan Reeves — "Don't let 'endpoint' grow teeth it doesn't need yet"

**Dan Reeves · Systems Minimalist**

I'll grant that named endpoints are better than inline model strings — they're reusable and they decouple "what model" from "where it's referenced." Accept this.

But I want a hard line at the boundary: **an endpoint is a model routing handle, not a credentials store.** The moment you add `api_key` or `base_url` or `timeout` fields, you've built infrastructure that has to be secured, versioned, and documented. That's three separate features disguised as one config field.

For v1: `id`, `provider`, `model`. Period. No auth fields. The auth story is: you set the right env vars for your provider, sdlc just passes them through.

**The provider field concern:** If we add `provider: anthropic` today and plan `provider: openai` later, we're implicitly promising cross-provider support. Tobias already killed executor switching in session 1 — the protocol mismatch with non-claude CLIs means we can't actually switch executors. Does the same apply to OpenAI?

Using OpenAI as a fallback would require spawning a different CLI with different stdin/stdout semantics than `claude`. No provider-agnostic executor exists in this codebase.

⚑  Decided: Endpoints in v1 are Anthropic-only. `provider` field is present but restricted to `anthropic`. Multi-provider support is explicitly deferred.

?  Open: Is multi-provider actually wanted here, or is this about multi-model within Anthropic (opus → sonnet → haiku)?

---

### Tobias Krenn — "What is a 'skill' and why does it matter?"

**Tobias Krenn · Skeptical Engineering Lead**

Jordan used "endpoint/skill" in a compound. I need to understand what "skill" means here before we design routing logic around it, because the implementation path forks dramatically.

**Interpretation A: Skill = sdlc ActionType**
The state machine has 20+ action types: `create_spec`, `implement_task`, `approve_review`, etc. This interpretation means: "run `create_spec` steps on Haiku (cheap, deterministic), run `implement_task` steps on Opus (complex reasoning)."

**Interpretation B: Skill = human-named category**
User creates named categories: "planning-tasks," "implementation-tasks." Assigns action types to categories. Assigns chains to categories. More flexible but more config surface.

**Interpretation C: Skill = the sdlc slash commands themselves**
`/sdlc-run`, `/sdlc-next`, `/sdlc-prepare`, etc. Less likely — these are command interfaces, not task classification.

**Interpretation D: Skill = milestone type or feature type**
Already handled by per-milestone chain override.

My read: Jordan most likely means Interpretation A (action types) with a simpler mental model — "planning stuff should use a cheaper model, coding stuff should use a smarter one." The ActionType framework already captures this perfectly.

⚑  Decided: "Skill" = ActionType in the sdlc context. The config field is named `action_chains` to make this explicit.

---

### Dan Reeves — "Server-side directive pre-read is actually clean"

**Dan Reeves**

Tobias raised the implementation concern about reading the directive before spawning. Let me think through whether that's expensive.

Reading `sdlc next --for <slug>` in the server before spawning just means:
1. Load the feature YAML from disk (already done by other handlers)
2. Run the classifier (pure in-memory logic, < 1ms)
3. Return the action type

This is exactly what `sdlc_get_directive` (the MCP tool) does — it's the same function path. The server can call the classifier directly via library code, not via subprocess.

The sequence for a feature run:
```
1. POST /api/runs/feature/:slug
2. Server: load Feature from .sdlc/features/<slug>/manifest.yaml
3. Server: classify → get ActionType (in memory, ~0ms)
4. Server: resolve model chain: action_chains[type] → milestone chain → default chain → hardcoded
5. Server: spawn_agent_run with resolved model
```

`QueryOptions` already has a `model: Option<String>` field (confirmed from code). The wire mechanism works.

⚑  Decided: Per-action-type routing works by classifying the feature in-server before spawning. No extra subprocess. Zero latency penalty.

---

### The Fallback Chain Design

The phrase "if x quota, then y quota, then z quota" is the most mechanically interesting part. Let me think through what "quota" means technically.

**Option 1: Static declaration** — user says "I have 50% Opus quota left." sdlc tracks this.
*Problem*: Users won't maintain quota declarations. Stale immediately.

**Option 2: Dynamic error detection** — claude CLI exits with a specific error pattern when rate limited. sdlc detects this and retries with the next endpoint.
*Problem*: Requires parsing error messages, brittle and version-dependent.

**Option 3: Stateful failure tracking** — sdlc tracks per-endpoint "last failure" with a reason code. If endpoint[0] failed with `rate_limit` or `overloaded` in the last N minutes, skip it and try [1]. The tracking state lives in `.sdlc/agent-state.yaml` (gitignored).

**Option 3 is the right approach.** It's automatic (no user maintenance), robust (doesn't need perfect error parsing — just "did it fail with a transient error?"), and decays naturally (failures expire after N minutes).

**The failure state store:**
```yaml
# .sdlc/agent-state.yaml  (gitignored)
endpoints:
  opus-primary:
    last_failure_at: 2026-03-03T04:50:00Z
    failure_reason: rate_limit
    cooldown_until: 2026-03-03T05:05:00Z
  sonnet-main:
    # no entry = healthy
```

**Chain resolution algorithm:**
```rust
fn resolve_endpoint(chain: &[EndpointId], state: &AgentState) -> Option<EndpointId> {
    for id in chain {
        if state.is_available(id) {  // cooldown_until < now
            return Some(id.clone());
        }
    }
    None  // all exhausted
}
```

**When all endpoints exhausted:** Emit SSE event `FeatureRunQuotaExhausted { slug, tried }`. Surface in frontend: "All model endpoints on cooldown. Next available: sonnet-main in ~1m."

**Cooldown TTL defaults:**
- `rate_limit` → 5 minutes (RPM resets quickly, daily token limits much longer)
- `overloaded` → 1 minute (transient load spike)
- Configurable per endpoint: `cooldown_minutes: 5`

**Mid-run quota hit:** Agent starts on Opus, 50 turns in, hits rate limit. Run exits with error. sdlc marks Opus on cooldown. Server retries the entire run with the next endpoint in chain. Full restart — agent starts fresh with a note in the prompt: "Previous attempt was interrupted. Continue from the beginning."

⚑  Decided: Fallback is automatic via stateful failure tracking in `.sdlc/agent-state.yaml` (gitignored). Cooldown TTL: 5min for rate_limit, 1min for overloaded. Mid-run failure triggers full restart with next endpoint.

⚑  Decided: `agent-state.yaml` is gitignored. Per-developer transient runtime state — doesn't belong in project git history.

---

### Pressure Test: Dynamic Case Matrix

**Case 1 — Happy path, single endpoint**
- Chain: `[opus-primary]`
- Run starts, opus responds, completes normally.
- Result: Normal. No fallback logic invoked.

**Case 2 — Primary hits rate limit, falls to secondary**
- Chain: `[opus-primary, sonnet-main]`
- Opus run exits with rate_limit error.
- `opus-primary` marked on cooldown (5 min).
- Run restarted with `sonnet-main`.
- Result: Success on sonnet. Run record shows "fallback used."

**Case 3 — Two concurrent milestones, different chains**
- Milestone A: chain `[opus-primary, sonnet-main]`
- Milestone B: chain `[haiku-cheap, sonnet-main]`
- Both run in parallel via `sdlc_run_wave`.
- Each `spawn_agent_run` call gets its chain from the milestone config.
- Result: No interference. Independent processes, independent cooldown state checks.

**Case 4 — Entire chain exhausted**
- Chain: `[opus-primary, sonnet-main]`, both on cooldown.
- `resolve_endpoint` returns `None`.
- Server returns `503 Service Unavailable`:
  `{ "error": "quota_exhausted", "cooldowns": { "opus-primary": "3m", "sonnet-main": "1m" } }`
- Frontend shows "All model endpoints on cooldown. Next available: sonnet-main in ~1m."
- Result: User-visible failure with clear recovery info.

**Case 5 — Per-action-type routing, cheap planning vs expensive coding**
- Config:
  ```yaml
  agent:
    action_chains:
      create_spec: [haiku-cheap, sonnet-main]
      create_design: [haiku-cheap, sonnet-main]
      implement_task: [opus-primary, sonnet-main]
      approve_review: [opus-primary, sonnet-main]
  ```
- Feature with `create_spec` action → server classifies → haiku chain.
- Feature with `implement_task` action → server classifies → opus chain.
- Result: ~$0.003 for spec write, ~$0.15 for implementation. Cost savings compound at scale.

**Case 6 — Quota recovery**
- `opus-primary` on cooldown at T=0.
- At T=5m, cooldown expires.
- Next run checks cooldown → available again.
- `resolve_endpoint` returns `opus-primary`.
- Result: Automatic recovery, no user action needed.

**Case 7 — Chain defined at milestone, no project default chain**
- Milestone has `agent_chain: [sonnet-main]`.
- Project config has no `default_chain`.
- Feature run: resolves milestone chain → uses sonnet.
- Result: Works. Project default chain is optional.

**Case 8 — No chain configured anywhere**
- Neither milestone nor project config has chain.
- Server falls back to hardcoded default: `claude-sonnet-4-6` (current behavior).
- Result: Zero behavior change for existing projects. Zero migration cost.

**Case 9 — Action type has no specific chain**
- `action_chains` defines `implement_task` only.
- Feature's next action is `create_spec`.
- `create_spec` not in `action_chains` → use milestone chain → use project default chain → use hardcoded fallback.
- Result: Graceful degradation through the full hierarchy.

**Case 10 — Endpoint renamed in config**
- User renames endpoint `opus-v1` → `opus-primary` in config.
- Old cooldown entry for `opus-v1` is now orphaned in agent-state.yaml.
- Orphaned entries are harmless (never looked up for current config).
- GC strategy: prune entries not referenced in current config on next state write.
- Result: No stale cooldowns blocking the new endpoint name.

**Case 11 — Budget cap interaction**
- `observability.daily_budget_usd: 50.0` already in config.yaml.
- Budget tracking is separate from quota fallback.
- When daily budget is hit, ALL runs refuse to start.
- Fallback chain does NOT help when budget is exhausted — that's a spending cap, not rate limit.
- Result: These two systems are independent. Budget cap = kill-switch. Quota fallback = routing helper.

**Case 12 — Team context, agent-state.yaml conflicts**
- Two developers use the same project repo.
- Each developer's sdlc server writes different cooldown states.
- `.sdlc/agent-state.yaml` is gitignored — each developer has their own local state.
- Result: No git conflicts. Per-developer cooldown tracking.

**Case 13 — Wave run, partial quota hit**
- `/sdlc-run-wave` spawns 5 features in parallel.
- Features 1, 2, 3 use opus-primary successfully.
- Feature 4 hits rate limit, opus-primary goes on cooldown.
- Feature 5 (which hasn't spawned yet): resolve_endpoint skips opus-primary (on cooldown), uses sonnet-main.
- Feature 4 retries on sonnet-main.
- Result: Wave continues at full throughput. No features blocked.

**Case 14 — Manual cooldown reset**
- Developer is testing and wants to retry opus immediately after a rate limit.
- UI: "Reset cooldown" button in the Endpoints page for each endpoint.
- API: `POST /api/agent/state/:id/reset`
- Result: Clears cooldown_until for the endpoint.

---

### Ben Hartley — Config Screen Design

**Ben Hartley · Developer Productivity UX Designer**

"A full config screen in the left" — in the sdlc frontend context, "the left" means a sidebar nav item that opens a dedicated right-side page (the same split pattern as milestones/features/roadmap).

A modal or drawer will feel cramped immediately for this configuration surface. A dedicated page is the right call.

**Sidebar nav placement:** New item between "Roadmap" and the footer. Icon: `Cpu` or `Settings2`. Label: "Endpoints."

**Page layout — two-pane:**

```
┌─ ENDPOINTS ────────────────────────────────────────────────────────────┐
│  ┌─ Endpoint List ──────────┐  ┌─ Selected: opus-primary ────────────┐ │
│  │                          │  │  Model:     claude-opus-4-6         │ │
│  │ ● opus-primary    [OK]   │  │  Provider:  anthropic               │ │
│  │ ● sonnet-main     [OK]   │  │  Status:    ✓ Available             │ │
│  │ ○ haiku-cheap  [cool 4m] │  │  Last used: 3 minutes ago          │ │
│  │                          │  │  Last fail: 18 min ago (rate_limit)│ │
│  │ [+ Add Endpoint]         │  │  [Reset Cooldown]                   │ │
│  └──────────────────────────┘  └────────────────────────────────────┘ │
│                                                                         │
│  ┌─ Default Chain ────────────────────────────────────────────────────┐│
│  │  ① opus-primary  →  ② sonnet-main  →  ③ haiku-cheap              ││
│  │  [↑↓ reorder]    [+ Add Step]    [Save Chain]                     ││
│  └────────────────────────────────────────────────────────────────────┘│
│                                                                         │
│  ┌─ Action Type Overrides ─────────────────────────────────────────────┐│
│  │  ACTION              CHAIN                                     DEL  ││
│  │  create_spec    → [haiku-cheap, sonnet-main]                   [×]  ││
│  │  implement_task → [opus-primary, sonnet-main]                  [×]  ││
│  │  approve_review → [opus-primary]                               [×]  ││
│  │  [+ Add Override]                                                   ││
│  └────────────────────────────────────────────────────────────────────┘│
└────────────────────────────────────────────────────────────────────────┘
```

**Status indicators:** Green dot = available. Yellow dot = cooldown. Red dot = permanent failure (if we add that state later). Cooldown countdown: "cool 4m" — updates live via SSE.

**Drag-to-reorder:** Chains are ordered lists. Drag-and-drop with keyboard fallback (up/down arrows in the list). A well-known pattern (Vercel environment variables have this exact UI).

**Per-milestone overrides:** NOT on this page. They live in the milestone detail pane. The milestone detail shows: "Agent Chain: [opus-primary, sonnet-main] (milestone override)" with inline edit.

**Edit endpoint:** Click endpoint in list → detail pane shows editable form. Click "Save" → `PATCH /api/agent/config` → config.yaml updated.

⚑  Decided: Dedicated "Endpoints" page reachable from sidebar. Two-pane layout. Default chain builder. Action type overrides table. Per-milestone overrides in milestone detail pane.

---

### Dana Cho — "Is this premature complexity?"

**Dana Cho · Product Skeptic**

The uncomfortable question: who actually has multiple providers with different quota limits they're managing today?

The scenario Jordan described implies a user who hits rate limits regularly, has configured fallback options, and has enough volume to make automatic fallback worthwhile vs. manual switching.

But I'll push back on myself: if you're running 20 features in parallel (run_wave), you **will** hit rate limits. The sdlc use case *is* high-volume autonomous execution. The rate limit case is not hypothetical — it's the expected behavior at scale. Automatic fallback that recovers without babysitting runs is directly valuable to the core workflow.

**On the config screen:** The 95% case (change model for a milestone) is covered by per-milestone chain override, which is 3 YAML lines. The config screen is for users who want to see the system-wide picture: which endpoints are healthy, what the default chain is, which action types route where. It's an observability surface as much as a configuration surface.

⚑  Decided (with reservation): The fallback chain is justified by the core sdlc use case (high-volume parallel autonomous execution). Config screen is necessary observability. Not premature.

?  Open (Dana's standing question): Can we ship the YAML config path first (no UI) and add the config screen in a follow-on feature within the same milestone? Yes — this should be the delivery sequence: backend first (endpoint model + retry logic), then UI.

---

### Architecture Summary

**Data model (sdlc-core):**
```rust
pub struct AgentEndpoint {
    pub id: String,
    pub provider: String,  // "anthropic" only for now
    pub model: String,
    pub cooldown_minutes: Option<u32>,  // default 5
}

pub struct AgentConfig {
    pub endpoints: Vec<AgentEndpoint>,
    pub default_chain: Vec<String>,  // endpoint ids
    pub action_chains: HashMap<String, Vec<String>>,  // action_type → [endpoint ids]
}
// Added to Config struct:
// pub agent: Option<AgentConfig>

// Added to Milestone struct:
// pub agent_chain: Option<Vec<String>>  // override only

pub struct EndpointState {
    pub last_failure_at: Option<DateTime<Utc>>,
    pub failure_reason: Option<String>,
    pub cooldown_until: Option<DateTime<Utc>>,
}

pub struct AgentState {
    pub endpoints: HashMap<String, EndpointState>,
}
// Stored at: .sdlc/agent-state.yaml (gitignored)
```

**Resolution chain (server):**
```
resolve_model(action_type, milestone, config) → Vec<EndpointId>:
  1. config.agent.action_chains[action_type]  // per-action override
  2. milestone.agent_chain                    // per-milestone override
  3. config.agent.default_chain               // project default
  4. ["claude-sonnet-4-6"]                    // hardcoded fallback
```

**Retry loop in spawn_agent_run:**
```
for endpoint in resolved_chain:
  if agent_state.is_available(endpoint):
    result = spawn_run(endpoint.model)
    if result == RateLimit or Overloaded:
      agent_state.mark_cooldown(endpoint, result.reason)
      continue  // try next
    else:
      return result
emit QuotaExhausted SSE
return 503
```

**New REST endpoints:**
- `GET /api/agent/config` → AgentConfig (read from config.yaml)
- `PATCH /api/agent/config` → update (write to config.yaml)
- `GET /api/agent/state` → AgentState (read from agent-state.yaml)
- `POST /api/agent/state/:id/reset` → clear cooldown for endpoint
- `GET /api/agent/state` also via SSE push when endpoint status changes

**New SSE events:**
- `AgentEndpointFallback { feature_slug, from_endpoint, to_endpoint }`
- `AgentEndpointCooldown { endpoint_id, cooldown_until }`
- `AgentEndpointRecovered { endpoint_id }`
- `AgentQuotaExhausted { feature_slug, tried: Vec<String> }`

**Frontend:**
- `EndpointsPage.tsx` — dedicated page, two-pane layout
- Sidebar: new nav item between Roadmap and footer
- MilestoneDetailPane: adds per-milestone chain override UI
- RunCard: shows "fallback used" badge and "quota exhausted" state

---

### Final Open Questions

?  Open: Should "Reset all cooldowns" be a single button on the Endpoints page, not just per-endpoint? Useful during development.

?  Open: Should the config screen also surface `observability.daily_budget_usd`? Natural grouping (all agent cost controls in one place). Low effort to include.

?  Open: Rate limit vs overloaded error detection — how robust is the claude CLI's exit behavior? Need to check `claude-agent` crate for error type handling. If the exit codes/stderr patterns are not stable, option 3 (stateful tracking) degrades to option 1 (static declaration). This is an implementation risk worth flagging.

?  Open: Confirm with Jordan that "skill" = ActionType is the right interpretation. The design assumes it, but the word "skill" is ambiguous in this codebase.
