---
session: 1
timestamp: 2026-03-03T06:00:00Z
orientation:
  current: "Audit complete. ThreadsPage is broken (no mobile hide logic). ToolsPage has no back button. Evolve/Guideline/Investigation are functional but use a different pattern than Ponder. Two targeted fixes needed."
  next: "Implement ThreadsPage mobile fix (15 min). Then ToolsPage back button + prevent auto-select on mobile. File as feature tasks."
  commit: "When ThreadsPage and ToolsPage fixes are tracked as implementation tasks and we agree not to unify all workspace pages under one pattern."
---

**jordan · Owner**
The threads layout is not mobile friendly

Need to improve all layouts like this - threads, ponder, guidelines (all Workspaces) and tools and any others like this

---

## Session 1 — Mobile Layout Audit

### Facilitator opening

The brief covers a wide surface. Before we plan any work, let's audit what's actually broken vs. what's just inconsistent. There's a big difference between "completely unusable on mobile" and "different pattern but functional." Jordan's frustration starts with Threads — let's validate why and then work outward.

Read all the relevant pages. Results:

---

**Ben Hartley · Developer Productivity UX**

I've looked at every page. The situation is clearer than it sounds. There are exactly two broken pages and three that just need consistency attention.

**ThreadsPage is genuinely broken.** This is the bug. The left pane has this:

```tsx
<div className="w-[280px] shrink-0 border-r border-border flex flex-col overflow-hidden md:flex md:w-[280px]">
```

The `md:flex` class does absolutely nothing — the element is already `flex`. There's no `hidden` prefix. On a mobile viewport you get two columns: 280px list + flex-1 detail, side by side. On a 390px iPhone screen that's a 280px list and a 110px "detail" pane. It's not just "not mobile friendly" — it's actively broken.

This is also why the symptom is Threads specifically. PonderPage, EvolvePage, GuidelinePage — these all have `showMobileDetail ? 'hidden md:flex' : 'flex'` logic. Threads somehow got skipped.

?  Open: Did Threads land before the mobile pattern was established, or was it written after and the pattern just wasn't applied?

---

**Tobias Krenn · Skeptical Engineering Lead**

Either way, the answer is the same: apply the pattern. The fix is three changes to ThreadsPage:

1. Left pane: add `slug ? 'hidden md:flex' : 'flex'` conditional
2. Right pane: add `slug ? 'flex flex-col' : 'hidden md:flex md:flex-col'` conditional
3. Back button in ThreadDetailPane header (already has the `onBack` prop plumbed — just needs a `md:hidden` back arrow)

That's a 15-minute fix. Done. Ship it.

The harder question is whether jordan wants a consistency pass across all workspace pages. Let me say something provocative: **the bottom sheets on Evolve/Guideline/Investigation are fine.** They work. Users can open the Files panel. Migrating them to PonderPage's tab-bar model is real work for marginal gain. Save that for a dedicated UX polish milestone.

⚑  Decided: EvolvePage, GuidelinePage, InvestigationPage are functional. Bottom sheet pattern is acceptable. No migration to tab bars needed now.

---

**Ben Hartley**

I'll push back slightly on Tobias. The inconsistency matters when users switch between pages. If you're in Ponder (tab bar) and then go to Evolve (header icon → bottom sheet), the interaction model has shifted under you. That's a small cost per session but it compounds.

But I agree with the prioritization. Fix the broken thing first. The consistency work is a separate milestone.

Where I do want us to address something: **ToolsPage has a subtle mobile bug too.** On load it auto-selects the first tool:

```tsx
} else if (data.length > 0 && !selectedNameRef.current) {
  setSelectedName(data[0].name)
}
```

On desktop that's fine — you always want something selected. On mobile, that means you land on `/tools` and immediately see the detail view, never the list. There's also no back button — no `onBack` handler anywhere. You're stuck.

The fix is slightly more nuanced: detect mobile breakpoint on first load and skip auto-select. Or add a back button that clears `selectedName`. Or switch ToolsPage to URL routing like the other pages.

---

**Tobias Krenn**

Simplest fix for ToolsPage: add a back button that sets `selectedName` to null, and skip auto-select when viewport is narrow. Tobias rule: don't switch to URL routing unless there's a reason to deep-link to a specific tool. There isn't one currently.

Actually — check the existing left pane logic:

```tsx
<div className={cn(
  'w-64 shrink-0 border-r border-border flex flex-col bg-card',
  selectedTool ? 'hidden md:flex' : 'flex',
)}>
```

ToolsPage *does* hide the list when a tool is selected — the hide logic is there. The problem is just that a tool is always selected on load (auto-select), so the list is always hidden. Fix: don't auto-select on mobile.

And then add a back button inside the tool detail panel (wherever that renders) that calls `setSelectedName(null)`.

---

**Ben Hartley**

Let me map the full impact of "no mobile handling" across all pages:

| Page | Status | Primary Issue | Fix Complexity |
|---|---|---|---|
| ThreadsPage | 🔴 Broken | Left pane never hides — both columns always visible | Low (15 min) |
| ToolsPage | 🟡 Partial | Auto-select means detail always shows; no back button | Low (20 min) |
| PonderPage | ✅ Good | Tab bar, back button, list/detail routing | Reference impl |
| EvolvePage | ✅ Functional | Bottom sheet works; back button exists | None needed |
| GuidelinePage | ✅ Functional | Bottom sheet works; back button exists | None needed |
| InvestigationPage | ✅ Functional | Bottom sheet works; back button exists | None needed |

The scope is narrower than the brief implies. "All layouts" makes it sound like six pages. It's two fixes.

---

**Dan Reeves · Systems Minimalist**

One more thing to check: the AppShell. The `main` element has `overflow-y-auto` and pages use `flex h-full overflow-hidden`. This pattern is fine on desktop — main has a constrained height from the flex parent, so h-full works. But check whether pages actually fill the viewport on mobile without scrolling past the BottomTabBar.

Looking at AppShell — there's no BottomTabBar in the codebase at all. The bottom tab bar (`BottomTabBar.tsx`) exists but I don't see it rendered in `AppShell.tsx`. The mobile header is there. But the bottom navigation — is it mounted? If it's not rendered, that's a navigation gap on mobile.

Wait — `AppShell.tsx` has `<AgentPanelFab />` at the bottom. Let me check if BottomTabBar is being rendered anywhere...

---

**Facilitator**

Checked: BottomTabBar is defined in `frontend/src/components/layout/BottomTabBar.tsx` and exported, but it is not rendered in `AppShell.tsx`. The `Sidebar` provides desktop navigation. Mobile navigation currently relies on the hamburger menu button in the mobile header and the sidebar overlay. The BottomTabBar component exists but is orphaned.

?  Open: Should BottomTabBar be rendered in AppShell for mobile? Or is the hamburger + overlay the intended mobile nav pattern?

---

**Ben Hartley**

That's a separate question from layout fixes. The hamburger works. It's not great, but it's functional. Don't pull that thread into this session — it deserves its own ponder.

Let's stay focused: what are the deliverables from this session?

---

**Tobias Krenn**

Two tasks:

1. **Fix ThreadsPage** — Apply PonderPage's `showMobileDetail` pattern. Left pane hides when slug is present. Back button in ThreadDetailPane. Right pane shows only when slug present on mobile. 15 minutes of code.

2. **Fix ToolsPage mobile back navigation** — Skip auto-select on mobile (detect `window.innerWidth < 768` or a media query). Add a back button that sets `selectedName(null)`. 20 minutes.

Don't open the consistency refactor for workspace pages now. That's a separate "workspace-ux-polish" feature.

⚑  Decided: Two targeted fixes (Threads + Tools). Workspace pages (Evolve/Guideline/Investigation) are functional. BottomTabBar question is a separate ponder.

---

**Ben Hartley**

Agreed. One thing to add to the ThreadsPage fix: the mobile tab bar inside the detail pane. PonderPage has Chat / Files / Team. Threads doesn't have Files (no workspace panel) or Team, but there's still a meaningful split: the thread list pane is navigation, and when you're in a thread on mobile you just need the conversation.

Actually — ThreadDetailPane already shows the full thread. There's no secondary panel. So the only change is list/detail switching with a back button. Even simpler than Ponder.

---

## Summary

**What's broken (critical):**
- ThreadsPage — both panes visible simultaneously on mobile. Fix: `showMobileDetail` pattern + back button.

**What's partial (functional but fixable):**
- ToolsPage — auto-select forces detail on mobile with no way back. Fix: skip auto-select on mobile + add back button.

**What's fine:**
- PonderPage — reference implementation. Full tab bar, back button, list/detail routing.
- EvolvePage / GuidelinePage / InvestigationPage — bottom sheet pattern works. Back button exists.

**What's a separate question:**
- BottomTabBar orphan — component exists but isn't rendered.
- Workspace page consistency (bottom sheet vs. tab bar) — separate milestone.

**Implementation scope:** Two files, ~35 lines of changes total. Fast, surgical.
