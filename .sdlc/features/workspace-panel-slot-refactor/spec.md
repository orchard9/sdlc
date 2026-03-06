# Spec: WorkspacePanel slot-based refactor

## Problem

`WorkspacePanel` currently contains 8 separate conditional branches (lines 101–153 in `WorkspacePanel.tsx`) that render phase-aware UI panels based on `kind` and `phase` props. Each branch is a hard-coded combination of `kind === '...' && phase === '...'`, resulting in:

- Tight coupling: the component must import and know about every domain-specific panel (`AreaCards`, `OutputGate`, `SynthesisCard`, `LensCards`, `EvolveOutputGate`, `GuidelineEvidenceCards`, `GuidelineOutputGate`)
- Poor extensibility: adding a new investigation kind or phase requires editing `WorkspacePanel` directly
- Reduced testability: each conditional branch is implicitly tested via the component rather than by composing panels externally
- Harder to read: the 50-line conditional block in the middle of the component obscures the panel's core responsibility (artifact list + navigation)

## Proposed Solution

Replace the 8-branch conditional with a `phasePanel` prop — an optional `ReactNode` that callers render and pass in. `WorkspacePanel` renders it verbatim in the dedicated slot, with no knowledge of `kind`, `phase`, or any investigation-specific components.

This is a pure refactor: no behavior changes, no UX changes, no business logic changes.

## Detailed Changes

### `WorkspacePanel` component (`frontend/src/components/ponder/WorkspacePanel.tsx`)

1. Remove props: `phase`, `kind`, `investigation` from the `Props` interface
2. Add prop: `phasePanel?: ReactNode`
3. Replace the 8 conditional JSX blocks (lines 101–153) with a single slot:
   ```tsx
   {phasePanel && (
     <div className="shrink-0 border-b border-border/40">
       {phasePanel}
     </div>
   )}
   ```
4. Remove all domain-specific imports: `AreaCards`, `OutputGate`, `SynthesisCard`, `LensCards`, `EvolveOutputGate`, `GuidelineEvidenceCards`, `GuidelineOutputGate`
5. Add `ReactNode` to React import

### Callers — pass phase panel content as a prop

Each workspace page/component that currently passes `kind`, `phase`, and `investigation` to `WorkspacePanel` must be updated to construct the appropriate `phasePanel` node inline and pass it as the `phasePanel` prop.

Affected callers (to be identified during implementation):
- Investigation pages / panels that render `WorkspacePanel` with `kind` / `phase` / `investigation`
- Ponder pages that render `WorkspacePanel`

Each caller already imports its own domain-specific panel components (or should). After this refactor, `WorkspacePanel` imports none of them.

## Acceptance Criteria

1. `WorkspacePanel` props interface contains `phasePanel?: ReactNode` and does NOT contain `phase`, `kind`, or `investigation`
2. `WorkspacePanel` imports no domain-specific panel components
3. All 8 phase/kind combinations continue to render identically (same DOM structure, same conditional logic) — verified by visual inspection or snapshot
4. All existing callers compile without TypeScript errors
5. No behavior change: artifact list, navigation, fullscreen modal, swipe handlers are unchanged
6. No new files added — all changes are edits to existing files

## Non-Goals

- No changes to individual phase panel components (`AreaCards`, etc.)
- No routing or URL changes
- No changes to state management
- No changes to SSE subscriptions
