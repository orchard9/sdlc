# Tasks: WorkspaceShell component

## T1: Create WorkspaceShell component

Create `frontend/src/components/layout/WorkspaceShell.tsx` with the `WorkspaceShell` component as specified in the design. The component accepts `listPane`, `detailPane`, `showDetail`, and optional `listWidth` props. Uses `cn` from `@/lib/utils`. Export named: `export function WorkspaceShell`.

## T2: Refactor PonderPage to use WorkspaceShell

Replace the outer `<div className="h-full flex flex-col overflow-hidden">` shell in `PonderPage` with `<WorkspaceShell showDetail={showMobileDetail} listPane={...} detailPane={...} />`. The AdvisoryPanel and NewIdeaModal overlays remain outside the shell (they are fixed overlays, not part of the two-pane layout).

## T3: Refactor EvolvePage to use WorkspaceShell

Replace the outer shell in `EvolvePage` with `<WorkspaceShell showDetail={showMobileDetail} listPane={...} detailPane={...} />`.

## T4: Refactor InvestigationPage to use WorkspaceShell

Replace the outer shell in `InvestigationPage` with `<WorkspaceShell showDetail={showMobileDetail} listPane={...} detailPane={...} />`.

## T5: Refactor GuidelinePage to use WorkspaceShell

Replace the outer shell in `GuidelinePage` with `<WorkspaceShell showDetail={showMobileDetail} listPane={...} detailPane={...} />`.

## T6: Verify TypeScript build passes

Run `cd frontend && npm run build` (or `npm run typecheck` if available) and confirm there are no TypeScript errors. Fix any type errors that arise from the refactor.
