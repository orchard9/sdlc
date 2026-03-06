# Design: WorkspacePanel slot-based refactor

## Overview

This is a pure internal refactor — no UI changes, no new screens, no UX changes. The visual output is identical before and after. A mockup is not applicable.

## Current Architecture

```
WorkspacePanel
├── props: artifacts, onClose, phase, kind, investigation
├── imports: AreaCards, OutputGate, SynthesisCard, LensCards,
│           EvolveOutputGate, GuidelineEvidenceCards, GuidelineOutputGate
└── 8 conditional JSX blocks (lines 101–153):
    ├── kind === 'root_cause' && phase === 'investigate' → <AreaCards>
    ├── kind === 'root_cause' && phase === 'output'     → <OutputGate>
    ├── kind === 'root_cause' && phase === 'synthesize' → <SynthesisCard>
    ├── kind === 'evolve' && phase === 'analyze'        → <LensCards>
    ├── kind === 'evolve' && (phase === 'paths'|'roadmap') → inline ArtifactContent
    ├── kind === 'evolve' && phase === 'output'         → <EvolveOutputGate>
    ├── kind === 'guideline' && phase === 'evidence'    → <GuidelineEvidenceCards>
    ├── kind === 'guideline' && (phase === 'principles'|'draft') → inline ArtifactContent
    └── kind === 'guideline' && phase === 'publish'     → <GuidelineOutputGate>
```

## Target Architecture

```
WorkspacePanel
├── props: artifacts, onClose, phasePanel? (ReactNode)
├── imports: none of the domain-specific panel components
└── single slot:
    └── {phasePanel && <div className="shrink-0 border-b border-border/40">{phasePanel}</div>}

Callers (InvestigationPage, PonderPage, etc.)
├── construct the phasePanel ReactNode using kind/phase/investigation
└── pass it as <WorkspacePanel phasePanel={...} ...>
```

## Change Plan

### Step 1 — Update `WorkspacePanel` props and template

In `frontend/src/components/ponder/WorkspacePanel.tsx`:

1. Import `ReactNode` from React
2. Replace `Props` interface:
   - Remove: `phase?: string`, `kind?: string`, `investigation?: InvestigationDetail`
   - Add: `phasePanel?: ReactNode`
3. Replace the 8 conditional blocks with one slot:
   ```tsx
   {phasePanel && (
     <div className="shrink-0 border-b border-border/40">
       {phasePanel}
     </div>
   )}
   ```
4. Remove unused imports: `AreaCards`, `OutputGate`, `SynthesisCard`, `LensCards`, `EvolveOutputGate`, `GuidelineEvidenceCards`, `GuidelineOutputGate`, `InvestigationDetail`

### Step 2 — Update all callers

Find every file that renders `<WorkspacePanel` with `kind`, `phase`, or `investigation` props.

For each caller, extract the previously-inlined conditional logic into a local `phasePanel` variable of type `ReactNode`, built using the same conditions that existed in `WorkspacePanel` before. Pass it as `phasePanel={phasePanel}`.

The callers already import the types needed (or will import them). Each domain-specific panel import migrates from `WorkspacePanel.tsx` to the caller file.

## Callers to Identify

Grep for usages:
```
frontend/src/**/*.tsx — search for "<WorkspacePanel" with kind= or phase= props
```

Expected callers based on codebase structure:
- `frontend/src/pages/InvestigationPage.tsx` (or similar investigation workspace page)
- `frontend/src/pages/PonderPage.tsx`
- Possibly `frontend/src/components/investigation/WorkspacePanel.tsx` variants

## No-Change Invariants

- Rendered DOM output is identical for all `(kind, phase)` combinations
- Navigation, fullscreen, pagination, swipe all unchanged
- TypeScript must compile cleanly (`tsc --noEmit`)
