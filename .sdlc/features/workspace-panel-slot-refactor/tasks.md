# Tasks: WorkspacePanel slot-based refactor

## T1 — Refactor WorkspacePanel to accept `phasePanel` slot

In `frontend/src/components/ponder/WorkspacePanel.tsx`:

1. Add `ReactNode` to React imports
2. Update `Props` interface:
   - Remove: `phase?: string`, `kind?: string`, `investigation?: InvestigationDetail`
   - Add: `phasePanel?: ReactNode`
3. Remove unused imports: `AreaCards`, `OutputGate`, `SynthesisCard`, `LensCards`, `EvolveOutputGate`, `GuidelineEvidenceCards`, `GuidelineOutputGate`, `InvestigationDetail`
4. Replace the 8 conditional JSX blocks (lines 101–153) with:
   ```tsx
   {phasePanel && (
     <div className="shrink-0 border-b border-border/40">
       {phasePanel}
     </div>
   )}
   ```
5. Update function signature: remove `phase`, `kind`, `investigation` from destructuring, add `phasePanel`

## T2 — Update InvestigationPage callers

In `frontend/src/pages/InvestigationPage.tsx` (2 usages):

Build a `phasePanel` ReactNode using the same conditional logic that was in `WorkspacePanel`. The `kind` is `'root_cause'` for InvestigationPage. Add required imports (`AreaCards`, `OutputGate`, `SynthesisCard`) if not already present.

Replace `phase={entry.phase} kind={entry.kind} investigation={entry}` with `phasePanel={phasePanel}` on both `<WorkspacePanel>` usages.

## T3 — Update EvolvePage callers

In `frontend/src/pages/EvolvePage.tsx` (2 usages):

Build a `phasePanel` ReactNode for evolve kind: `LensCards` for `analyze`, inline `ArtifactContent` for `paths`/`roadmap`, `EvolveOutputGate` for `output`. Add required imports if not already present.

Replace `phase={entry.phase} kind={entry.kind} investigation={entry}` with `phasePanel={phasePanel}` on both `<WorkspacePanel>` usages.

## T4 — Update GuidelinePage callers

In `frontend/src/pages/GuidelinePage.tsx` (2 usages):

Build a `phasePanel` ReactNode for guideline kind: `GuidelineEvidenceCards` for `evidence`, inline `ArtifactContent` for `principles`/`draft`, `GuidelineOutputGate` for `publish`. Add required imports if not already present.

Replace `phase={entry.phase} kind={entry.kind} investigation={entry}` with `phasePanel={phasePanel}` on both `<WorkspacePanel>` usages.

## T5 — TypeScript clean compile

Run `cd frontend && npx tsc --noEmit` — zero errors required.
