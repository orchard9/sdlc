# QA Plan: WorkspacePanel slot-based refactor

## Scope

Pure structural refactor — no behavior change. QA focuses on:
1. TypeScript compilation (zero type errors)
2. Correct rendering of all `(kind, phase)` combinations via the new slot
3. Unchanged artifact list, navigation, fullscreen, and swipe behaviors

## Test Approach

### 1. TypeScript compile check (automated)

```bash
cd frontend && npx tsc --noEmit
```

Pass criterion: zero errors.

### 2. Visual regression — phase panel rendering

For each `(kind, phase)` combination, verify the correct panel renders in the WorkspacePanel header slot:

| kind | phase | Expected phase panel |
|---|---|---|
| root_cause | investigate | AreaCards |
| root_cause | synthesize | SynthesisCard |
| root_cause | output | OutputGate |
| evolve | analyze | LensCards |
| evolve | paths | ArtifactContent (paths.md) |
| evolve | roadmap | ArtifactContent (roadmap.md) |
| guideline | evidence | GuidelineEvidenceCards |
| guideline | principles | ArtifactContent (toc.md) |
| guideline | draft | ArtifactContent (guideline-draft.md) |
| guideline | publish | GuidelineOutputGate |
| ponder (no kind) | any | no phase panel (empty slot) |

Verification method: manual visual inspection in the running dev server, or `tsc --noEmit` plus code review confirming the phasePanel logic in each caller mirrors the removed WorkspacePanel conditions exactly.

### 3. Props interface check

Confirm `WorkspacePanel` Props type no longer contains `phase`, `kind`, or `investigation`:
- TypeScript will enforce this — any caller still passing old props will be a compile error

### 4. Import hygiene

Confirm `WorkspacePanel.tsx` no longer imports any domain-specific panel:
- `AreaCards`, `OutputGate`, `SynthesisCard`, `LensCards`, `EvolveOutputGate`, `GuidelineEvidenceCards`, `GuidelineOutputGate`
- `InvestigationDetail` type

### 5. PonderPage callers (no phasePanel)

PonderPage passes `<WorkspacePanel artifacts={entry.artifacts} />` with no `kind/phase/investigation`. After refactor, it still passes no `phasePanel`. Verify no phase panel appears — the slot renders nothing when `phasePanel` is undefined.

## Pass Criteria

- [ ] `npx tsc --noEmit` exits 0
- [ ] All 8 phase/kind combinations show correct panel header
- [ ] PonderPage shows no phase panel (unchanged)
- [ ] No runtime errors in browser console on any investigation/evolve/guideline/ponder page
- [ ] WorkspacePanel artifact list, navigation, fullscreen, and pagination work identically
