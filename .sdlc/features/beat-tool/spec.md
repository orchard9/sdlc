# Spec: beat-tool — proof-of-concept agentic tool

## Problem

The sdlc project has a powerful state machine and growing tool ecosystem. But no single tool demonstrates all three shared primitives working together in one agentic flow: reading project state (`_shared/sdlc.ts`), recruiting and invoking a reasoning agent (`_shared/agent.ts`), and streaming structured progress via NDJSON. Without a working end-to-end example, each new agentic tool re-invents patterns independently.

Additionally, there is no lightweight "pulse check" for whether the project is on track. Developers and agents alike need a fast way to get a leadership-level verdict: is this project heading in the right direction, is it drifting, or is it off-course? And on a weekly cadence, they need a top-5 prioritized check-in list to focus async work.

## Goal

Build the `beat` tool as a proof-of-concept that validates the full agentic tool pattern end-to-end:

1. Reads project state (features, milestones, vision) via `_shared/sdlc.ts`
2. Recruits a leadership agent (CTO/CPO lens) if not already present via `_shared/agent.ts`
3. Invokes that agent to reason over the state
4. Emits a verdict: `on-track | drifting | off-course` with concerns
5. Persists to `.sdlc/beat.yaml` for trend tracking
6. Supports `--week` mode to produce a top-5 check-in list from previous beats

## Input Schema

```typescript
{
  scope: 'project' | '<domain>' | 'feature:<slug>'
  mode: 'evaluate' | 'week'
}
```

- `scope: 'project'` — evaluates the entire project against VISION.md
- `scope: '<domain>'` — evaluates features matching a domain tag or milestone slug
- `scope: 'feature:<slug>'` — evaluates a single feature
- `mode: 'evaluate'` — runs a fresh evaluation, writes to beat.yaml
- `mode: 'week'` — reads previous evaluations from beat.yaml, returns top-5 weekly check-in items

## Output Schema

```typescript
{
  verdict?: 'on-track' | 'drifting' | 'off-course'   // evaluate mode
  score?: number                                        // 0–100 alignment score
  concerns?: string[]                                   // list of concern strings
  week_items?: WeekItem[]                               // week mode: top-5 items
  beat_id?: string                                      // id of the written beat record
}

interface WeekItem {
  priority: number    // 1–5
  item: string        // one-liner
  feature?: string    // slug if feature-specific
}
```

## Streaming

The tool streams NDJSON progress events to stdout during execution. Each event is a JSON object on its own line:

```
{"event":"gathering","message":"Reading project state..."}
{"event":"recruiting","message":"Loading CTO/CPO agent..."}
{"event":"evaluating","message":"Agent reasoning over 14 features..."}
{"event":"writing","message":"Persisting beat record beat-001..."}
{"event":"done","result":{...}}
```

The final line is always `{"event":"done","result":<ToolResult>}`. Callers that don't need streaming can read only the last line.

## Persistence

Evaluations are appended to `.sdlc/beat.yaml`:

```yaml
beats:
  - id: beat-001
    scope: project
    mode: evaluate
    timestamp: 2026-03-03T12:00:00Z
    verdict: on-track
    score: 78
    concerns:
      - "3 features in implementation >14 days — staleness risk"
      - "No QA plan for citadel-webhook-handler"
```

Week mode reads all recent beats (configurable window, default 14 days) and synthesizes a prioritized check-in list.

## Shared Primitives Required

This tool requires two new `_shared/` modules that will be usable by all future agentic tools:

### `_shared/sdlc.ts`
Reads project state by shelling out to the `sdlc` CLI:
- `readVision(root)` — reads VISION.md
- `readFeatures(root)` — calls `sdlc feature list --json`
- `readMilestones(root)` — calls `sdlc milestone list --json`
- `readFeatureDetail(root, slug)` — calls `sdlc feature show <slug> --json`

### `_shared/agent.ts`
Recruits and invokes a named agent:
- `ensureAgent(root, slug, roleDescription)` — uses `sdlc ponder recruit` if agent file missing, then loads it
- `runAgent(agentPath, prompt, opts)` — invokes the agent file via the claude CLI with `--print`, captures output
- Returns structured output parsed from the agent response

## Agent Lens

For `scope: 'project'`, the beat tool recruits a CTO/CPO persona with the following characteristics:
- Evaluates features against VISION.md alignment, not individual correctness
- Identifies drift: features heading toward solved problems that don't appear in the vision
- Identifies off-course signals: key milestones stalled, no momentum on strategic objectives
- Weekly mode: prioritizes by business risk, not implementation order

For `scope: 'feature:<slug>'`, the lens shifts to a tech lead perspective: correctness, task completeness, blockers, and timeline risk.

## Non-Goals

- This tool does NOT write code or execute fix actions — it is read-only + advisory
- This tool does NOT replace `sdlc next` for implementation directives
- This tool does NOT require an external LLM API key — it uses the local claude CLI
- Week mode does NOT run a new evaluation; it only reads existing beat records

## Success Criteria

1. `sdlc tool run beat --input '{"scope":"project","mode":"evaluate"}'` completes without error and writes a beat record
2. NDJSON progress events stream correctly during execution
3. Week mode reads from beat.yaml and returns 5 prioritized items
4. `_shared/sdlc.ts` and `_shared/agent.ts` are importable by other tools
5. `sdlc tool sync` correctly picks up the beat tool metadata
6. The tool is documented in `tools.md` after sync

## Milestone Context

This feature is part of the v27 agentic tool suite milestone. It is the integration proof-of-concept that validates the shared primitive design before other agentic tools are built on top of it.
