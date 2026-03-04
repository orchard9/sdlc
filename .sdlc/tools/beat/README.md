# Beat — Project Pulse Check

Evaluates project direction against VISION.md using a recruited leadership agent. Produces a verdict with concerns, and persists to `.sdlc/beat.yaml` for trend tracking.

## Usage

```bash
# Evaluate the full project against VISION.md
sdlc tool run beat --input '{"scope":"project","mode":"evaluate"}'

# Evaluate a single feature
sdlc tool run beat --input '{"scope":"feature:my-feature","mode":"evaluate"}'

# Get top-5 weekly check-in items from recent beats
sdlc tool run beat --input '{"scope":"project","mode":"week"}'
```

## Input

| Field | Type | Required | Description |
|---|---|---|---|
| `scope` | string | yes | `"project"`, a milestone slug, or `"feature:<slug>"` |
| `mode` | `"evaluate"` \| `"week"` | yes | Evaluate runs fresh; week reads history |

## Output

**evaluate mode:**
```json
{
  "ok": true,
  "data": {
    "verdict": "on-track",
    "score": 78,
    "concerns": ["3 features stalled in implementation", "No QA plan for citadel-webhook-handler"],
    "beat_id": "beat-001"
  }
}
```

**week mode:**
```json
{
  "ok": true,
  "data": {
    "week_items": [
      { "priority": 1, "item": "3 features stalled in implementation" },
      { "priority": 2, "item": "no qa plan for citadel-webhook-handler", "feature": "citadel-webhook-handler" }
    ]
  }
}
```

## Streaming

The tool streams NDJSON progress events to stdout during execution:

```
{"event":"gathering","message":"Reading project state (14 features, 6 milestones)..."}
{"event":"recruiting","message":"Loading cto-cpo-lens agent..."}
{"event":"evaluating","message":"Agent evaluating 14 features against VISION.md..."}
{"event":"writing","message":"Persisting beat record to .sdlc/beat.yaml..."}
{"event":"done","result":{"ok":true,"data":{...}}}
```

## How it Works

1. **Gather** — reads VISION.md and feature/milestone state via `_shared/sdlc.ts`
2. **Recruit** — ensures a `cto-cpo-lens` or `tech-lead-lens` agent exists via `_shared/agent.ts`
3. **Evaluate** — invokes the agent with vision + feature context, parses JSON verdict
4. **Persist** — appends the evaluation to `.sdlc/beat.yaml`

## Persistence

Evaluations accumulate in `.sdlc/beat.yaml`:

```yaml
beats:
  - id: beat-001
    scope: project
    mode: evaluate
    timestamp: 2026-03-03T12:00:00Z
    verdict: on-track
    score: 78
    concerns:
      - 3 features stalled in implementation
```

## Setup

No setup required. The tool will recruit the leadership agent automatically on first run. Requires:
- `claude` CLI available in PATH (for agent invocation)
- `sdlc` CLI available in PATH (for state reading)
