# Root Cause: Investigation Sessions Don't End

## Diagnosis

The investigation prompt in `start_investigation_chat` (runs.rs:1468-1659) is incomplete for two of the three investigation kinds:

### What's broken

**Root-cause and Evolve are missing output-phase instructions.**

The prompt tells agents:
- Root-cause sequence: `triage` → `investigate` → `synthesize` → `output`
- Evolve sequence: `survey` → `analyze` → `paths` → `roadmap` → `output`

But it never says what to DO in the `output` phase — no artifact specification, no `--status complete` call.

**Guideline is fine** — it has a complete finish sequence (lines 1605-1610 in runs.rs) including `sdlc investigate update {slug} --status complete`.

### What happens today

1. Agent starts a root-cause session
2. It advances through triage → investigate → synthesize
3. It advances to `output` phase
4. ...crickets. No instructions. Agent may try to invent something, loop, or just stop without completing.
5. Investigation stays `in_progress` forever — no `complete` status, no findings document

### Secondary: missing output artifact specs

- Root-cause output: no canonical output artifact defined (what IS the deliverable?)
- Evolve output: same — no `action-plan` or equivalent artifact defined

### Frontend: mechanically correct

- `InvestigationDialoguePanel` subscribes to `investigation_run_completed` SSE ✓
- `handleInvestigationEvent` clears runState on completion ✓
- `spawn_agent_run` ALWAYS emits InvestigationRunCompleted when agent finishes ✓
- Stuck UI can only happen if SSE event is missed (connection drop) — secondary concern

## Fix

Edit the prompt in `start_investigation_chat` (runs.rs) to add output-phase instructions for root-cause and evolve.

### Root-cause output phase

Write `findings.md` (canonical root-cause document), then:
```bash
sdlc investigate update <slug> --confidence <0-100>
sdlc investigate update <slug> --output-ref "findings.md"
sdlc investigate update <slug> --status complete
```

### Evolve output phase

Write `action-plan.md` (chosen path + concrete next steps), then:
```bash
sdlc investigate update <slug> --status complete
```

⚑  Decided: Root cause is missing output-phase instructions in the server-side prompt.
⚑  Decided: Fix is targeted edit to runs.rs — no schema changes needed.
?  Open: Should we also add a mount-time check in InvestigationDialoguePanel to recover from missed SSE events?
