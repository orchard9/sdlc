---
session: 1
timestamp: 2026-03-04T07:15:00Z
orientation:
  current: "Audit complete — validated all findings against live code, corrected stale artifacts, change plan is ready"
  next: "Create milestone and features from the validated change plan — three tracks"
  commit: "Met — the change plan is concrete, line-level, and validated against actual code"
---

**jordan · Owner**
workspace layout improvements

1. for all layouts we need to make it so the chat bar is always stuck to the bottom (be careful about doing this change properly, really think it through)
2. for the medium sized layout/tablet layout we need to collapse earlier, either make a tablet layout or use the mobile earlier
3. do a review of the layouts and make sure the code is clean and dialed in

---

## Code Audit — Validating Previous Artifacts

Previous session produced `layout-audit.md` and `change-plan.md`. This session validated every finding against the actual codebase. Several findings were stale or wrong.

### Corrections to layout-audit.md

**Bug 1 (AppShell `main` missing `min-h-0`) — ALREADY FIXED.** The actual code at `AppShell.tsx:144` reads:
```
<main className="flex-1 min-h-0 overflow-y-auto">
```
`min-h-0` is present. The audit was either written against an older version or preemptively.

**Bug 3 (BottomTabBar covers mobile content) — PHANTOM BUG.** `BottomTabBar` is **dead code** — defined in `frontend/src/components/layout/BottomTabBar.tsx` but never imported or rendered anywhere in the app. Zero imports. The `pb-11` fix in the change plan is unnecessary.

**AgentPanelFab at `bottom-[56px]`** — positioned to clear a BottomTabBar that doesn't exist. The 56px offset is vestigial. With no BottomTabBar, the FAB could sit at `bottom-4` like a normal mobile FAB. However, since this is on PonderPage (which has its own in-page tab bar ~44px tall), the FAB needs to clear that. More on this below.

### Confirmed Real Issues

**InputBar missing `shrink-0` — CONFIRMED.** Both `DialoguePanel.tsx` InputBar (lines 129, 146) and `InvestigationDialoguePanel.tsx` InputBar (lines 112, 129) have no `shrink-0` on their root `<form>` / `<div>`. In a flex-col container, this means under flex pressure the input bar could compress instead of maintaining its natural height. The flex chain does propagate `min-h-0` correctly so the stream div should absorb overflow, but `shrink-0` makes the intent explicit and safe.

**Breakpoint at `md:` (768px) — CONFIRMED.** Every layout-mode switch across all files uses `md:`. Files affected:
- `AppShell.tsx` — sidebar, mobile header, panel button (5 instances)
- `PonderPage.tsx` — desktop/mobile content, floating nav, tab bar (6 instances)
- `EvolvePage.tsx` — back button, workspace toggle, desktop sidebar, mobile bottom sheet, pane visibility (7 instances)
- `InvestigationPage.tsx` — identical pattern (7 instances)
- `GuidelinePage.tsx` — identical pattern (7 instances)
- `AgentPanel.tsx` — aside visibility (1 instance)
- `AgentPanelFab.tsx` — FAB + drawer (3 instances)

Zero `lg:` classes exist in any of these files.

---

## Track 1 — Chat Bar Pinning

**CSS Layout Specialist perspective:**

The flex chain from root to InputBar is actually well-constructed:

```
div.flex.h-[100dvh].overflow-hidden          AppShell root (row)
  div.flex-1.flex.flex-col.overflow-hidden   Content column
    main.flex-1.min-h-0.overflow-y-auto      Main area
      div.h-full.flex.flex-col.overflow-hidden  Page root
        div.flex-1.flex.min-h-0              Page inner row
          div.flex-1.min-w-0.min-h-0        Detail pane
            div.h-full.flex.flex-col.min-h-0  DialoguePanel
              div.shrink-0                   Header row
              div.flex-1.overflow-y-auto     Stream (scrollable)
              <form>                         InputBar ← NO shrink-0
```

The chain is correct — `min-h-0` propagates at every flex level. The only gap is InputBar's missing `shrink-0`. Without it, the InputBar *could* be compressed if the stream div's content somehow creates upward flex pressure. In practice this rarely fires because the stream div has `overflow-y-auto` (scrolls instead of pushing). But `shrink-0` is the correct defensive declaration.

**The fix is surgical:**

In `DialoguePanel.tsx`, the InputBar is an inline component (not a separate file). Two render paths:
- Line 129: `<div className="flex items-center gap-2 ...">` (running state)
- Line 146: `<form className="flex items-end gap-2 ...">` (idle state)

In `InvestigationDialoguePanel.tsx`, same pattern:
- Line 112: `<div className="flex items-center gap-2 ...">` (running state)
- Line 129: `<form className="flex items-end gap-2 ...">` (idle state)

Add `shrink-0` to all four root elements. That's it.

⚑  Decided: Add `shrink-0` to all four InputBar root elements. No AppShell changes needed — `min-h-0` is already there, `pb-11` is not needed (BottomTabBar is dead code).

---

## Track 2 — Breakpoint Shift (md → lg)

**Mobile UX perspective:**

At 768px (iPad mini portrait), the current layout forces a three-column desktop view: sidebar (224px) + content + agent panel. That leaves ~544px for content, which is absurdly cramped for a workspace UI with dialogue panels, workspace sidebars, and list views.

At 1024px (`lg`), the desktop layout gets ~800px for content, which is workable. iPad portrait devices (768-1024px) get the mobile single-column view, which is correct — they're touch devices that work better with the mobile UX.

**The mechanical change:** Replace `md:` → `lg:` for all layout-mode switching classes. The change plan's list is accurate. ~30 class replacements across 8 files.

**Critical distinction:** Only layout-MODE switches change. Inner responsive tweaks (e.g., `md:flex-row` for visual polish inside a panel, `hidden sm:block` for slug display) stay at their current breakpoints. The rule: if it controls whether you see mobile single-column vs. desktop multi-column, it's `md:` → `lg:`. Everything else stays.

⚑  Decided: Replace `md:` → `lg:` for layout-mode switches only. Interior visual tweaks stay at `md:` or `sm:`.

**AgentPanelFab `bottom-[56px]`:** Once BottomTabBar breakpoint concern is gone (it's dead code), this should change to `bottom-4` on the FAB. On PonderPage, the in-page tab bar is in-flow (not fixed), so the FAB doesn't need to clear it — they're in different stacking contexts. Actually, the FAB is `fixed` so it overlays the in-page tab bar. But the tab bar is at the bottom of the flex column, and the FAB is `fixed bottom-[56px]`. The FAB should stay above the PonderPage tab bar when it exists. PonderPage's tab bar is ~44px tall at the bottom of its flex column.

? Open: Should AgentPanelFab position change? The `bottom-[56px]` is only relevant when PonderPage is showing its in-page tab bar. On other pages there's no bottom bar. A simple fix: `bottom-4` by default. PonderPage can position its floating nav accordingly. But actually, the FAB only shows when there are agent runs — it's an overlay that should be unobstructed. `bottom-4` is fine. The PonderPage floating entry nav buttons are at `bottom-16` (64px), which would still be above a `bottom-4` FAB.

⚑  Decided: Change AgentPanelFab from `bottom-[56px]` to `bottom-4`. Clean up the vestigial BottomTabBar clearance.

---

## Track 3 — Code Cleanup

### Dead Code: BottomTabBar

`BottomTabBar.tsx` is never imported or rendered. It should be deleted. It was designed for an app-level mobile navigation but the app currently uses the sidebar (hamburger) for mobile nav. Keeping dead code creates confusion — the previous audit artifacts spent significant analysis on a component that doesn't exist in the rendered UI.

⚑  Decided: Delete `BottomTabBar.tsx`. It's dead code.

### PonderPage In-Page Tab Bar

PonderPage has its own mobile tab bar (Chat/Files/Team) at line 578. This is an in-page component, not a global nav. It's in-flow (`shrink-0 flex`) inside the mobile flex column, which is correct. No changes needed here.

### Flex Chain Documentation

The change plan suggests adding HEIGHT CONTRACT comments to dialogue panels. This is low-value — the chain is well-structured and adding comments to every level creates maintenance noise. The chain is: `h-[100dvh]` → `flex-col overflow-hidden` → `flex-1 min-h-0` → `h-full flex-col min-h-0` → `flex-1 overflow-y-auto` → `shrink-0` (InputBar). Anyone reading the code can trace this.

⚑  Decided: No HEIGHT CONTRACT comments. The code is self-documenting if InputBar gets `shrink-0`.

### WorkspaceLayout Extraction

The change plan mentions extracting a shared `<WorkspaceLayout>` component since PonderPage, EvolvePage, InvestigationPage, and GuidelinePage share the same desktop/mobile pattern. This is real — the four pages have nearly identical structure:

```
Desktop: hidden lg:flex flex-1 min-h-0
  ├── Dialogue (flex-1 min-w-0)
  ├── Resize handle (optional)
  └── Workspace panel (shrink-0)
Mobile:  lg:hidden flex-1 min-h-0 flex flex-col
  ├── Tab content (flex-1)
  └── Tab bar (shrink-0)
```

But extraction should come AFTER the breakpoint shift, when the code is stable. Mark as future work, not part of this change.

⚑  Decided: WorkspaceLayout extraction is deferred. Do mechanical fixes first.

### `overflow-y-auto` on `main`

`AppShell.tsx:144` has `overflow-y-auto` on the `<main>` element. Pages use `overflow-hidden` on their root divs, so the main scroll container never actually scrolls in practice (pages contain their own scrolling). This is harmless but slightly misleading. Changing it to `overflow-hidden` would make the intent clearer — main is a constraint box, not a scroll container.

? Open: Should `main` use `overflow-hidden` instead of `overflow-y-auto`? Low priority — both work because pages set `h-full overflow-hidden` on their roots. But `overflow-hidden` on main would be more correct since we never want the shell itself to scroll.

---

## Validated Change Plan (Corrected)

### 1. InputBar `shrink-0` (4 elements, 2 files)

**DialoguePanel.tsx:**
- Line 129: `<div className="flex items-center gap-2 ...">` → add `shrink-0`
- Line 146: `<form className="flex items-end gap-2 ...">` → add `shrink-0`

**InvestigationDialoguePanel.tsx:**
- Line 112: `<div className="flex items-center gap-2 ...">` → add `shrink-0`
- Line 129: `<form className="flex items-end gap-2 ...">` → add `shrink-0`

### 2. Breakpoint shift `md:` → `lg:` (~30 replacements, 8 files)

Only layout-mode switches. Files:
- `AppShell.tsx` (5 locations)
- `PonderPage.tsx` (6 locations)
- `EvolvePage.tsx` (7 locations)
- `InvestigationPage.tsx` (7 locations)
- `GuidelinePage.tsx` (7 locations)
- `AgentPanel.tsx` (1 location)
- `AgentPanelFab.tsx` (3 locations)
- AppShell panel-open button (1 location)

Plus corresponding `pb-0` prefix changes if any exist (none found — `pb-11` was never added).

### 3. Dead code cleanup

- Delete `BottomTabBar.tsx`
- Change AgentPanelFab FAB from `bottom-[56px]` to `bottom-4`
- PonderPage floating nav: adjust `bottom-16` to `bottom-12` (since FAB moves down)

### 4. Optional: `main` overflow

- `AppShell.tsx:144`: consider `overflow-hidden` instead of `overflow-y-auto`

### Implementation order

1. Delete BottomTabBar.tsx (clean slate)
2. InputBar `shrink-0` additions (4 elements)
3. Breakpoint `md:` → `lg:` mechanical replacements
4. AgentPanelFab position fix
5. Optional: `main` overflow change
6. Visual testing at 768px and 1024px viewports
