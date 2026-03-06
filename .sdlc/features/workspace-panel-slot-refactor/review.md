# Code Review: WorkspacePanel slot-based refactor

## Summary

This is a pure structural refactor — no behavior changes, no UX changes. The implementation is correct and complete.

## Files Changed

- `frontend/src/components/ponder/WorkspacePanel.tsx` — core refactor
- `frontend/src/pages/InvestigationPage.tsx` — caller updated
- `frontend/src/pages/EvolvePage.tsx` — caller updated
- `frontend/src/pages/GuidelinePage.tsx` — caller updated

## Findings

### 1. WorkspacePanel.tsx — Props interface (PASS)

`Props` now contains `phasePanel?: ReactNode` and does NOT contain `phase`, `kind`, or `investigation`. The slot renders correctly:

```tsx
{phasePanel && (
  <div className="shrink-0 border-b border-border/40">
    {phasePanel}
  </div>
)}
```

Domain-specific imports (`AreaCards`, `OutputGate`, `SynthesisCard`, `LensCards`, `EvolveOutputGate`, `GuidelineEvidenceCards`, `GuidelineOutputGate`, `InvestigationDetail`, `InvestigationArtifact`) are fully removed. The linter also moved `formatBytes` to an import from `@/lib/utils` (pre-existing pattern in the codebase), which is a cleanup bonus.

Note: a `mediaBaseUrl?: string` prop was added by a concurrent change (ponder-binary-image-support feature). This is additive and does not conflict with this refactor.

### 2. InvestigationPage.tsx — caller (PASS)

The `EntryDetailPane` component correctly builds `phasePanel` for `root_cause` kind:
- `investigate` → `<AreaCards>`
- `output` → `<OutputGate>`
- `synthesize` → `<SynthesisCard>`

Both `<WorkspacePanel>` usages (desktop and mobile sheet) pass `phasePanel={phasePanel}`. Old props (`phase`, `kind`, `investigation`) are removed.

### 3. EvolvePage.tsx — caller (PASS)

The `EntryDetailPane` correctly builds `phasePanel` for `evolve` kind:
- `analyze` → `<LensCards>`
- `paths`/`roadmap` → inline `<ArtifactContent>` (with null guard on artifact content)
- `output` → `<EvolveOutputGate>`

Both `<WorkspacePanel>` usages pass `phasePanel={phasePanel}`.

### 4. GuidelinePage.tsx — caller (PASS)

The `EntryDetailPane` correctly builds `phasePanel` for `guideline` kind:
- `evidence` → `<GuidelineEvidenceCards>`
- `principles`/`draft` → inline `<ArtifactContent>` (with null guard)
- `publish` → `<GuidelineOutputGate>`

Both `<WorkspacePanel>` usages pass `phasePanel={phasePanel}`.

### 5. PonderPage.tsx — no-op (PASS)

PonderPage passes `<WorkspacePanel artifacts={entry.artifacts} />` with no `phasePanel` — the slot renders nothing. Unchanged and correct.

### 6. TypeScript compile (PASS)

`npx tsc --noEmit` exits 0 with no errors.

### 7. Conditional padding removed from slot wrapper (NOTE — accepted)

The original 8-branch code applied different padding classes to the slot wrapper for different `(kind, phase)` combinations (some had `px-0 py-0`, others had no extra padding). The new unified slot wrapper uses a single `shrink-0 border-b border-border/40` class. The callers that previously needed `px-0 py-0` on the outer div (`AreaCards`, `LensCards`, `GuidelineEvidenceCards`) now simply have no outer wrapper padding — the panel components manage their own internal padding. This is visually equivalent since those components render their own padding. Accepted — no change needed.

## Verdict

APPROVED. All spec acceptance criteria met:
1. `phasePanel?: ReactNode` present in Props; `phase`, `kind`, `investigation` absent
2. No domain-specific imports in WorkspacePanel
3. All phase/kind combinations covered in callers
4. TypeScript clean compile confirmed
5. No behavior change
6. No new files created
