# Extraction Strategy: Shared Workspace Components

## Principle: Shared Shell, Composed Specialization

Unify the **UI shell**, not the **data model**. The shell is where duplication costs us (bugs, inconsistency, maintenance). The data model differences are load-bearing.

## Components to Extract

### 1. `WorkspaceShell` — Page Layout
The two-pane list/detail pattern shared by all 4 workspace pages.

```typescript
interface WorkspaceShellProps {
  kind: string;                              // route prefix + API discriminator
  statusTabs: Tab[];                         // filter tabs with counts
  renderRow: (entry: any) => ReactNode;      // entry list item
  renderDetail: (entry: any) => ReactNode;   // right pane content
  renderCreate: () => ReactNode;             // creation modal/form
  emptyState: ReactNode;                     // no-selection placeholder
}
```

**Eliminates:** 3 copy-pasted investigation page files, PonderPage shell code

### 2. `UnifiedDialoguePanel` — Conversation Thread
Session rendering, input bar, auto-scroll, SSE subscription.

```typescript
interface DialogueConfig {
  headerSlot: ReactNode;                     // TeamRow+Orientation OR PhaseStrip
  startChat: (slug: string, msg: string) => Promise<void>;
  stopChat: (slug: string) => Promise<void>;
  sseChannel: 'ponder' | 'investigation';   // adapter pattern, no backend change
  mcpLabel: string;                          // tool call card label
  emptyAction?: ReactNode;                   // 'Start from brief' button (ponder only)
}
```

**Eliminates:** 2 duplicate dialogue panels with identical sub-components

### 3. `CreateWorkspaceModal` — Creation Form
Title + slug + configurable fields.

```typescript
interface CreateWorkspaceModalProps {
  kind: string;
  additionalFields?: FormField[];            // scope, context, etc.
  onCreate: (data: any) => Promise<void>;
}
```

**Eliminates:** 4 duplicate creation forms

### 4. `WorkspacePanel` Refactor — Slot-Based
Replace 8-branch `kind+phase` conditional with a `phasePanel` prop.

```typescript
interface WorkspacePanelProps {
  // existing props...
  phasePanel?: ReactNode;                    // workspace-specific phase content
}
```

**Eliminates:** God-component conditional branching

### 5. `utils/slug.ts` — Utility Extraction
`titleToSlug` currently exists in 4 files.

**Eliminates:** 4 copies of the same function

## What Stays Workspace-Specific

- TeamRow + OrientationStrip (Ponder)
- PhaseStrip (Investigation types)
- AreaCards, SynthesisCard, OutputGate (Root Cause)
- LensCards, EvolveOutputGate (Evolve)
- GuidelineEvidenceCards, GuidelineOutputGate (Guideline)

## What We Do NOT Do

- ✗ Unify PonderEntry and InvestigationEntry data models
- ✗ Create a god-component that knows about all workspace types
- ✗ Change backend SSE event structure (yet)
- ✗ Big-bang refactor — each step is independently shippable

## Implementation Order (incremental)

1. Extract `titleToSlug` → `utils/slug.ts`
2. Build `WorkspaceShell` page layout component
3. Refactor `WorkspacePanel` to slot-based
4. Build `UnifiedDialoguePanel`
5. Build `CreateWorkspaceModal`
6. Migrate pages one at a time (Ponder, Investigation, Evolve, Guideline)
7. Delete old duplicate files
