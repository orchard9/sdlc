---
session: 1
timestamp: 2026-03-04T07:15:00Z
orientation:
  current: "Mapped all duplication across 4 workspace types. Identified 3-layer extraction strategy: shell, dialogue, config-driven specialization. Key tension is whether to unify data models or keep them separate behind a shared UI contract."
  next: "Design the concrete component interfaces — WorkspaceShell props, UnifiedDialoguePanel config object, and the backend trait or generic route handler"
  commit: "When we have a component inventory with props/interfaces that cover all 4 workspace types without any god-object or mega-switch patterns"
---

**jordan · Owner**
ponder, root cause, and evolve, and all workspaces should use the same core components

like adding ideas, the conversation thread, artifact viewer, etc

---

**Facilitator**
This is a significant observation. I've done a thorough audit of the current frontend and backend — the duplication is worse than it looks from the outside. Let me bring in the right people.

I'm channeling **The Component Architect** (composition and interface design), **Ben Hartley** (developer UX, cognitive load), and **Dan Reeves** (systems minimalist — will push back on over-abstraction).

Let me present the findings and let them react.

---

**Facilitator · Current State Briefing**
See `current-state-audit.md` for the full map. Headlines:

1. **3 copy-pasted page components** (InvestigationPage, EvolvePage, GuidelinePage) — identical two-pane list/detail shell, differing only in `kind` string and route prefix
2. **2 duplicate dialogue panels** — DialoguePanel (Ponder) and InvestigationDialoguePanel share every internal sub-component (InputBar, McpCallCard, WorkingPlaceholder, scroll logic) verbatim
3. **WorkspacePanel** already attempted unification but became an 8-branch conditional god-component
4. **Backend has parallel route files** — roadmap.rs and investigations.rs with identical handler shapes
5. **Data models genuinely differ** — PonderEntry has teams/tags/commitment, InvestigationEntry has kind/phase/type-specific fields

---

**Component Architect**
The duplication map is clear. Let me identify what's actually composable here. I see three layers:

**Layer 1 — WorkspaceShell (pure chrome)**
The two-pane layout with status tabs, entry list, entry detail, mobile bottom sheet. This is identical across all four pages. It should be ONE component that takes:
- `kind: string` — route prefix and API discriminator
- `statusTabs: Tab[]` — the filter tabs
- `renderEntryRow: (entry) => ReactNode` — slot for entry rendering
- `renderDetail: (entry) => ReactNode` — slot for the right pane
- `createAction: () => void` — what happens when you click "New"

Every page file collapses to ~20 lines of configuration.

**Layer 2 — UnifiedDialoguePanel (conversation infrastructure)**
The session thread, input bar, auto-scroll, SSE subscription, send/stop lifecycle. All of this is identical. The only differences are:
- Which header strip to render (TeamRow+OrientationStrip vs PhaseStrip)
- Which API to call (startPonderChat vs startInvestigationChat)
- Which SSE channel to listen on

This should be ONE component with a config object:
```typescript
interface DialogueConfig {
  headerSlot: ReactNode;
  startChat: (slug: string, message: string) => Promise<void>;
  stopChat: (slug: string) => Promise<void>;
  sseChannel: 'ponder' | 'investigation';
  mcpLabel: string;
}
```

**Layer 3 — Workspace-specific panels (the genuine differences)**
Phase strips, team rows, area cards, lens cards, evidence cards, output gates — these remain workspace-specific components. They plug into the shell and dialogue panel as slots. No unification needed; they're the domain-specific leaves.

---

**Ben Hartley**
I agree with the three-layer structure, but I want to push on the user experience implications.

The reason these workspaces *feel* similar to Jordan is because they ARE similar from the user's perspective. You open a workspace, you see a list, you pick one, you see a conversation thread on the left and artifacts on the right. The interaction pattern is identical regardless of whether you're pondering an idea or investigating a root cause.

But here's the thing — **the consistency itself is a feature**. Users learn the pattern once and it transfers. If we unify the components, we also lock in that consistency going forward. Any improvement to the dialogue panel (better scroll behavior, richer session blocks, inline artifact previews) lands in all four workspace types simultaneously. Right now, a bug fix in DialoguePanel doesn't propagate to InvestigationDialoguePanel.

My concern is the 8-branch conditional in WorkspacePanel. That's the wrong kind of unification. It's a shared component that *knows about* every workspace type. When you add workspace type #5, you have to edit WorkspacePanel. That's the opposite of composable.

⚑ Decided: WorkspacePanel should use a slot/children pattern, not conditional branches. Each workspace type passes its own phase-specific panel as a prop.

---

**Dan Reeves**
Hold on. Before we build three new generic components, let me ask the uncomfortable question: **do we even need four workspace types?**

Ponder, Root Cause, Evolve, Guideline — they all have: a title, a status, sessions, artifacts, an orientation strip (or phase strip, which is structurally the same thing). The differences are:
- Ponder has teams (but could any workspace benefit from thought partners?)
- Investigations have phases (but ponder could have phases too — "exploring" IS a phase)
- The type-specific fields (confidence, lens_scores, etc.) are just metadata

What if there's ONE workspace type with a `kind` discriminator and optional fields? The backend already does this — InvestigationEntry uses `kind` to discriminate root_cause/evolve/guideline. Why not extend that to include `ponder`?

---

**Component Architect**
Dan raises a real question but I think the answer is "not yet, maybe never." Here's why:

The **data models** genuinely serve different purposes:
- Ponder's `committed_to` and `committed_at` link to milestones — that's a lifecycle concept that doesn't exist in investigations
- Investigation's `phase` field drives agent behavior — the classifier uses it to determine what artifact to produce next
- Ponder's `team.yaml` is a first-class team model — importing that into a generic workspace would mean every workspace type needs to handle team CRUD, even if most don't use it

The cost of a single god-model is worse than the cost of parallel models with a shared UI contract. We've seen this — WorkspacePanel's 8-branch conditional is exactly what happens when you unify too eagerly. You get a component that *knows about everything* instead of one that *composes with anything*.

? Open: Should ponder eventually gain phases? (e.g., exploring → converging is already phase-like)

? Open: Should investigations gain team support? (thought partners in root-cause analysis could be valuable)

---

**Ben Hartley**
I want to side with the Component Architect here. Unify the **UI shell**, not the **data model**. The shell is where the duplication actually costs us — bugs, inconsistency, maintenance burden. The data model differences are load-bearing.

But I'd add one more extraction target: **the creation flow**. Right now there are four creation forms (NewIdeaModal, NewInvestigationForm, NewEvolveForm, NewGuidelineForm) that are nearly identical. A `CreateWorkspaceModal` with a `fields: FormField[]` config would eliminate those duplicates too.

---

**Facilitator**
Let me synthesize. I'm seeing convergence on a clear strategy.

## The Strategy: Shared Shell, Composed Specialization

### Extract these shared components:

**1. `WorkspaceShell`** — the two-pane list/detail page layout
- Status tab bar with count badges
- Scrollable entry list with configurable `renderRow`
- Detail pane with configurable `renderDetail`
- Mobile bottom-sheet for workspace panel
- "New" button with configurable creation flow
- Route management (list vs detail views)
- `titleToSlug` utility (currently 4 copies!)

**2. `UnifiedDialoguePanel`** — the conversation thread
- Session loading and rendering via `SessionBlock` (already shared)
- Input bar with send/stop
- Auto-scroll with manual override
- SSE subscription (parameterized by channel)
- McpCallCard (parameterized by label)
- Working/loading placeholder
- **Header slot** for workspace-specific strips (TeamRow, PhaseStrip, etc.)

**3. `CreateWorkspaceModal`** — the creation form
- Title input with auto-slug derivation
- Configurable additional fields (scope, context, etc.)
- Kind-aware API dispatch

**4. `WorkspacePanel` refactor** — slot-based instead of conditional
- Keep the artifact list and viewer (already shared)
- Replace 8-branch conditional with a `phasePanel: ReactNode` prop
- Each workspace type renders its own phase-specific panel

### Keep these workspace-specific:

- `TeamRow` + `OrientationStrip` (Ponder only, for now)
- `PhaseStrip` (Investigation types)
- `AreaCards`, `SynthesisCard`, `OutputGate` (Root Cause)
- `LensCards`, `EvolveOutputGate` (Evolve)
- `GuidelineEvidenceCards`, `GuidelineOutputGate` (Guideline)

### Backend approach:

**Don't merge the data models.** Keep PonderEntry and InvestigationEntry separate.

**Do extract shared route handlers.** The session list/get/capture handlers are identical — extract a generic handler function that takes a `dir: &Path` parameter (matching workspace.rs's pattern). The CRUD handlers differ enough to stay separate.

⚑ Decided: Shared UI shell, composed specialization — NOT a unified data model.

⚑ Decided: WorkspacePanel uses slot-based composition, not conditional branching.

⚑ Decided: Backend data models stay separate; route handler duplication can be reduced with generic helpers.

? Open: Should we introduce a `WorkspaceKind` enum in the frontend that covers all four types, or keep ponder as a separate concept from investigations?

? Open: The SSE event channel split (ponder vs investigation) — should we unify to a single `workspace` channel with a `kind` discriminator, or keep them separate?

---

**Dan Reeves**
The SSE question matters. Right now the frontend has `onPonderEvent` and `onInvestigationEvent` as separate callbacks in `useSSE`. If we unify the dialogue panel, it needs one subscription mechanism. Two options:

1. **Keep separate channels, adapter in UnifiedDialoguePanel** — the panel accepts `sseChannel: 'ponder' | 'investigation'` and internally subscribes to the right one. Simple, no backend change.

2. **Unify to `onWorkspaceEvent`** — backend emits `WorkspaceRunStarted { kind, slug, session }` etc. Cleaner long-term, but requires backend SSE changes.

I'd say option 1 first (minimize blast radius), option 2 as a follow-up if we add more workspace types.

⚑ Decided: Start with adapter approach (option 1) for SSE. Unify the channel later if needed.

---

**Component Architect**
One more thing. The `titleToSlug` function exists in 4 files. That's a `utils/slug.ts` extraction. Dead simple, high value, do it first as a warmup.

Also — when we build `WorkspaceShell`, we should look at whether `PonderPage.tsx` fits the same pattern or is genuinely different. From the audit, PonderPage has a few extras (session preview in list, tags display) but the shell structure is the same. If we make `renderRow` a slot, those extras just go in the ponder-specific row renderer.

---

**Facilitator · Summary**

The strategy is clear. Three layers of extraction:

| Layer | Component | Eliminates |
|-------|-----------|------------|
| Page shell | `WorkspaceShell` | 3 duplicate page files + PonderPage shell |
| Conversation | `UnifiedDialoguePanel` | 2 duplicate dialogue panels |
| Creation | `CreateWorkspaceModal` | 4 duplicate creation forms |
| Artifact viewer | `WorkspacePanel` refactor | 8-branch conditional |
| Utility | `utils/slug.ts` | 4 copies of titleToSlug |

**What NOT to do:**
- Don't unify PonderEntry and InvestigationEntry data models
- Don't create a god-component that knows about all workspace types
- Don't change the backend SSE event structure (yet)

**Implementation order:**
1. Extract `titleToSlug` to `utils/slug.ts` (warmup, proves the pattern)
2. Build `WorkspaceShell` — the page layout component
3. Refactor `WorkspacePanel` to slot-based composition
4. Build `UnifiedDialoguePanel` — the conversation component
5. Build `CreateWorkspaceModal` — the creation form
6. Migrate each page (Ponder, Investigation, Evolve, Guideline) one at a time
7. Delete the old duplicate files

Each step is independently shippable. No big-bang refactor.
