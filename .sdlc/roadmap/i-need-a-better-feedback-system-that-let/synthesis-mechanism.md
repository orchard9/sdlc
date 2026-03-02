# Synthesis Mechanism — How Comments Fold Into the Core Element

## The Problem Synthesis Solves

Without synthesis, a thread degrades into a flat list of opinions — the core element (body) never evolves. Synthesis is the mechanism that makes the thread "living".

## Model: Versioned Body + Incorporated Flag

When synthesis runs:
1. Agent reads `body-vN.md` (current body)
2. Agent reads all comments where `incorporated: false`
3. Agent produces `body-v(N+1).md` — revised body that incorporates comment insights
4. Manifest: `body_version` incremented
5. Comments: `incorporated: true` on all absorbed comments

## What Synthesis Preserves

- ⚑ Original body is **never overwritten** — body-v1.md always exists
- ⚑ Every synthesis version is persisted — full audit trail
- ⚑ Comments are marked `incorporated: true` — not deleted
- ⚑ Attribution: synthesis-created versions can note "synthesized from comments C3, C4, C5"

## Trust Model

The versioned approach solves the trust problem (cf. Linear's failed synthesis):
- UI shows "Last synthesized from 3 comments at [time]"
- Version history is one click — see exactly what changed
- Revert to any previous version is one action
- Agent attribution is explicit: "agent:synthesizer updated body"

## Synthesis Triggers (V2 design)

Three trigger options:
1. **Manual** — user clicks "Synthesize" button; agent runs
2. **Threshold** — auto-triggers when N unincorporated comments accumulate
3. **On demand via CLI** — `sdlc thread synthesize <slug>`

Recommendation: start with manual (option 1 + 3). Auto-triggers are opaque and erode trust.

## Agent Prompt Shape (V2)

```
Current core element:
<body-vN.md contents>

New comments since last synthesis:
C3: [author] "comment text"
C4: [author] "comment text"

Task: Revise the core element to incorporate the insights from these comments.
Preserve the original intent. Where there is disagreement, surface it clearly.
Output: revised Markdown for the updated core element.
```