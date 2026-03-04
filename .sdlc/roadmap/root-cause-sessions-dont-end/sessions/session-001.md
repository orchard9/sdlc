---
session: 1
timestamp: 2026-03-03T22:30:00Z
orientation:
  current: "Root cause identified and fixed — missing output-phase instructions in investigation prompt"
  next: "Deploy/restart server so the new prompt takes effect; validate with a real root-cause investigation"
  commit: "Fix is deployed and a root-cause investigation reaches 'complete' status end-to-end"
---

**jordan · Owner**
root cause sessions dont end

look through root cause and evolve and make sure the flows are tied up and finish

---

## Session 1 — Root Cause Analysis + Fix

### Context Load

Loaded the investigation chat prompt from `crates/sdlc-server/src/routes/runs.rs` (`start_investigation_chat`, lines 1468–1715). Read the frontend lifecycle (`InvestigationDialoguePanel`, `SseContext`, `useSSE`). Traced the full run lifecycle through `spawn_agent_run`.

---

### The Problem

The investigation flow supports three kinds: `root_cause`, `evolve`, and `guideline`.

**Guideline is complete.** It has a full output sequence (lines 1605–1610 in runs.rs):
```bash
sdlc investigate update <slug> --output-ref ".sdlc/guidelines/<slug>.md"
sdlc investigate update <slug> --principles-count <N>
sdlc investigate update <slug> --status complete
```

**Root-cause and evolve are broken.** The prompt defines their phase sequences:
- Root-cause: `triage` → `investigate` → `synthesize` → `output`
- Evolve: `survey` → `analyze` → `paths` → `roadmap` → `output`

But it says NOTHING about what to do in the `output` phase. There is no:
- Canonical output artifact specification (`findings.md` for root-cause, `action-plan.md` for evolve)
- `sdlc investigate update --status complete` instruction

**Result**: Agents reach the `output` phase and don't know what to produce or when to stop. The investigation stays `in_progress` indefinitely.

---

### Frontend: Not the Culprit

Traced the frontend lifecycle:
- `handleInvestigationEvent` in `InvestigationDialoguePanel` correctly handles `investigation_run_completed` → clears `runState`
- `spawn_agent_run` ALWAYS emits `InvestigationRunCompleted` when the agent finishes (success or error)
- SSE routing is correct end-to-end

The frontend is mechanically sound. The stuck UI would only happen if an SSE event was missed due to a connection drop — this is a secondary concern, not the primary bug.

---

### The Fix (implemented)

Added a "Completing the investigation" section to the `start_investigation_chat` prompt in `runs.rs`, immediately before the session log step.

**Root-cause `output` phase**: Agents now write `findings.md` with a structured template (Finding, Evidence, Confidence, Recommended Next Steps), then call:
```bash
sdlc investigate update <slug> --confidence <0-100>
sdlc investigate update <slug> --output-ref "findings.md"
sdlc investigate update <slug> --status complete
```

**Evolve `output` phase**: Agents now write `action-plan.md` with a structured template (Chosen Path, Why, Action Steps, What We Are Not Doing), then call:
```bash
sdlc investigate update <slug> --status complete
```

Both fixes mirror the guideline's existing completion pattern.

**Compile verified**: `cargo build -p sdlc-server` passes clean.

---

### Remaining Open Question

?  Open: Should `InvestigationDialoguePanel` check on mount whether a run is currently active (poll `GET /api/runs` or similar)? If the SSE connection drops mid-run and the `investigation_run_completed` event is missed, the UI is stuck in "session in progress..." until page reload. Low-priority but worth a task.

⚑  Decided: Primary fix is prompt-only — no schema changes, no frontend changes needed.
⚑  Decided: Root-cause and evolve now have canonical output artifacts (`findings.md` and `action-plan.md`).
⚑  Decided: Both flows now end with `--status complete`, consistent with guideline.
