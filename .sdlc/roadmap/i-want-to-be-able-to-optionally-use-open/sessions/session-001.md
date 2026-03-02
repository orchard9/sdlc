---
session: 1
timestamp: 2026-03-02T19:00:00Z
orientation:
  current: "Two-concern request: (1) model selection per milestone — clearly implementable via milestone manifest + spawn_agent_run lookup; (2) OpenCode executor switching — architecturally harder due to protocol mismatch. Minimum viable path is model selection only."
  next: "Decide scope: model-only v1 vs model+executor. If model-only, design the `agent` field in Milestone struct and wire spawn_agent_run to read it."
  commit: "Agreement on scope boundary (model vs executor), config location (milestone manifest), and the lookup mechanism in spawn_agent_run."
---

**jordan · Owner**
i want to be able to optionally use open code for different milestones - and then choose the model. currently we default to claude sonnet. it can be scoped to a milestone

---

## Session 1 — Model Selection & Executor Config Per Milestone

### Opening orientation

This is a fresh ponder on a two-part request: (1) choose which AI model drives a milestone's agent runs, and (2) optionally switch the executor from Claude Code to OpenCode per milestone. Currently the server always spawns the `claude` binary with no model override (the claude default is `claude-sonnet-4-6`).

I want to pull in **Dan Reeves**, **Felix Wagner**, and **Tobias Krenn** here — this is exactly the kind of "add config to a data model" decision that warrants their challenge.

---

### Interrogating the brief

Before reaching for a config field, I want to understand what's actually driving this. There are a few distinct motivations that could explain it:

- **Cost**: Opus costs significantly more than Sonnet. Milestone X is complex architecture work where Opus earns its price. Milestone Y is a batch of routine CRUD features where Sonnet is plenty.
- **Capability**: Some milestones involve ambiguous, multi-constraint decisions where a smarter model produces meaningfully better output.
- **Executor diversity**: The project already generates OpenCode command files to `~/.opencode/command/`. Wanting to actually *run* features through OpenCode (rather than just have commands there) makes sense.
- **Vendor lock-in hedge**: Not wanting all agent runs hardcoded to Claude.

The simplest forcing question: **Is there a milestone right now that you wish were running on a different model?** If yes, what would the improvement look like? If no, this is speculative.

---

### Dan Reeves speaks — "Do you need a config field or an env var?"

**Dan Reeves · Systems Minimalist**

Before we touch `milestone.rs`, let me check whether existing mechanisms do the job.

`sdlc_query_options` currently returns `model: None`. The `build_command` function in `claude-agent/src/process.rs` only adds `--model` if it's `Some`. So today, the model is whatever `claude` defaults to.

**Option A (no data model change):** Export `SDLC_MODEL=claude-opus-4-6` before `sdlc run` or before starting the server for a particular session. That's zero code change. The server already passes `opts.env` to child processes — we'd just need `sdlc_query_options` to check `std::env::var("SDLC_MODEL")` and use it as the default.

**Option B (milestone config):** Add `agent: { model: "..." }` to the `Milestone` struct. `spawn_agent_run` for a feature would need to look up which milestone the feature belongs to, then apply the model override.

Option A is one-line-of-Rust and lives entirely in operational context (env vars). Option B is a data model change + a reverse lookup (feature → milestone) that currently doesn't exist in the server routes.

My default stance: prove Option A is insufficient before reaching for Option B.

?  Open: Is there a scenario where two concurrent milestones with different models would both be running at the same time? An env var can't express that.

---

### Felix Wagner responds — "The lookup problem is real but solvable"

**Felix Wagner · Developer Tooling Architect**

Dan's concurrent milestone scenario is exactly the case that breaks the env var approach. If you're running two waves in parallel — one on Opus, one on Sonnet — an env var serializes what should be parallel.

The data model question is: **where does the agent config live and what's the lookup path?**

Looking at the `Milestone` struct, there's no `agent` field today. The `Milestone` doesn't know which model to use. And the `spawn_agent_run` call sites don't have milestone context — `start_run(slug)` just takes the feature slug.

For the lookup to work, we need either:
1. Store agent config on `Milestone`, then when spawning a feature run, find which milestone the feature belongs to and inherit the config.
2. Store agent config on `Feature` directly — more granular but noisier.

I'd lean option 1. The natural scope Jordan described is "scoped to a milestone." Features inherit from their milestone.

**Proposed data shape** (milestone manifest addition):
```yaml
agent:
  model: claude-opus-4-6    # optional, inherits project default if absent
  executor: claude           # optional, defaults to "claude"
```

**Proposed lookup**: In `start_run`, after getting the feature slug, scan milestones to find which one contains this slug. This is a linear scan over milestones directory, cheap since there are rarely > 20.

⚑  Decided: Config belongs in milestone manifest. Feature inherits from milestone. No per-feature override needed in v1.

---

### Tobias Krenn pushes back — "The OpenCode story is the real risk here"

**Tobias Krenn · Skeptical Engineering Lead**

Model selection is straightforward and I have no objection. Add the field, wire the lookup. Twenty lines of Rust. Done.

But **OpenCode executor switching** is where I pump the brakes.

Here's the technical reality: `claude-agent` speaks a very specific protocol — `--output-format stream-json --input-format stream-json`. This is how the server drives agents: it spawns `claude` as a subprocess, sends a JSON message on stdin, reads JSONL from stdout, parses tool calls, emits SSE events. The entire `claude-agent` crate is built around this.

OpenCode (opencode.ai, by SST) is a TUI — a terminal user interface. It does not expose a programmatic stdin/stdout JSONL protocol. You can't drive it the way you drive `claude`. It's designed for a human at a terminal, not for a server spawning it as a subprocess.

So "use OpenCode as executor" in the server context means one of:
1. **OpenCode adds a non-interactive mode** — does `opencode --print` or similar exist? If so, does it speak the same stream-json protocol? Probably not.
2. **We build an adapter** — a shim that translates OpenCode's protocol to the one `claude-agent` expects. This is a significant investment.
3. **We interpret it differently** — maybe "use OpenCode" means "run the feature via a CLI session launched in the user's terminal," which is a completely different UX model (not server-driven at all).

?  Open: What does "use OpenCode" actually mean in Jordan's context? Is it "run agent tasks through OpenCode's TUI in my terminal"? Or "use OpenCode's API programmatically"?

My strong recommendation: **Scope v1 to model selection only**. Add `agent.model` to milestone manifest. Defer `agent.executor` until we understand what "OpenCode executor" actually means technically.

---

### Dan Reeves — "Agreed on scope, but what about a simpler default override?"

**Dan Reeves**

If the use case is "I want Opus for this milestone" then the feature request is legitimate and Felix's design is right.

But I want to call out: the *project-level default* is also worth configuring. Right now there's no way to say "this whole project runs on Sonnet 4.6." That's implicit. A project-level `agent.model` in `config.yaml` with per-milestone override seems like the complete picture.

**Two-level config hierarchy:**
```yaml
# .sdlc/config.yaml
agent:
  model: claude-sonnet-4-6   # project default

# .sdlc/milestones/v12-hard-work/manifest.yaml
agent:
  model: claude-opus-4-6     # milestone override
```

`sdlc_query_options` checks: milestone config → project config → hardcoded fallback.

This is clean. It's the standard override pattern. And it covers the env var use case too (you could still override with env var at the top of the hierarchy).

⚑  Decided: Two-level hierarchy — project default in config.yaml, milestone override in manifest.yaml.

---

### Exploring the OpenCode angle further

Let me check what `opencode` CLI actually supports. The sdlc codebase already generates `~/.opencode/command/sdlc-*.md` files — so commands run *within* OpenCode. But that's the user running OpenCode and typing `/sdlc-run`. The request might mean: "I want the agent runs that the server spawns to use OpenCode's models/API."

If that's the intent — using OpenCode as a *user-facing tool*, not a subprocess driver — then the feature isn't about `spawn_agent_run` at all. It's about which tool the human uses to execute `/sdlc-run` commands. The server doesn't get involved.

But if Jordan means the server should spawn OpenCode instead of Claude when running features… that's the protocol problem Tobias identified.

?  Open: Does OpenCode have a non-interactive mode? `opencode run <prompt>`? If yes, what's the output format?

The most honest thing to say in this session: **we need a spike** to answer the OpenCode question. Model selection doesn't need it. OpenCode executor needs it.

---

### Synthesis: What we've decided and what remains open

**⚑  Decided: Model selection is v1 scope**
- Add `agent: { model: "..." }` field to `Milestone` struct (optional, skip_serializing_if None)
- Add `agent: { model: "..." }` field to project `Config` struct (optional)
- Lookup chain: `spawn_agent_run` for feature → find milestone → merge config → pass to `sdlc_query_options`
- The reverse lookup (feature slug → milestone) is a scan of `.sdlc/milestones/*/manifest.yaml` and is cheap

**⚑  Decided: Two-level hierarchy**
- Project-level default in `.sdlc/config.yaml`
- Milestone-level override in `.sdlc/milestones/<slug>/manifest.yaml`
- Feature inherits from milestone; no per-feature override in v1

**⚑  Decided: `executor` field is deferred**
- The field can be reserved as `executor: Option<String>` in the struct for future use
- No execution logic for non-claude executors until protocol compatibility is confirmed
- A spike task should be logged to investigate OpenCode's non-interactive mode

**?  Open: OpenCode protocol compatibility**
- Does `opencode` support a stream-json or similar programmatic non-interactive mode?
- If not, what does "use OpenCode for a milestone" actually mean to Jordan in practice?
- This question must be answered before any executor switching can be implemented

**?  Open: Milestone-to-feature reverse lookup**
- Currently no function in sdlc-core to look up "which milestone contains feature X"
- This is needed for `spawn_agent_run` to find the milestone-level agent config
- Implementation: scan `.sdlc/milestones/` looking for manifest.yaml files that contain the feature slug in their `features` list

**?  Open: Should `sdlc feature run <slug>` CLI also respect this?**
- The CLI `sdlc run` command (if it exists) would also benefit from reading the model config
- Worth noting as a follow-on

---

### Minimal feature scope

This should be a tight feature: `milestone-agent-config`

```
As a developer, I can add an `agent.model` field to a milestone manifest
so that agent runs for features in that milestone use the specified model,
overriding the project default and the hardcoded fallback.
```

Implementation surface:
1. `sdlc-core/src/milestone.rs` — add `AgentConfig` struct + `agent: Option<AgentConfig>` to `Milestone`
2. `sdlc-core/src/config.rs` or wherever `Config` lives — add `agent: Option<AgentConfig>` to project config
3. New function: `fn milestone_for_feature(root, feature_slug) -> Option<Milestone>` in milestone.rs
4. `sdlc-server/src/routes/runs.rs` — `sdlc_query_options` (or a wrapper) reads milestone+project agent config
5. CLI: `sdlc milestone set-model <slug> <model>` (optional quality-of-life, not blocking)

That's a focused, testable change. The OpenCode executor question stays open for a follow-on spike.
