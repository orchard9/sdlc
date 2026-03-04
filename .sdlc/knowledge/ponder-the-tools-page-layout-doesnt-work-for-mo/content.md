---
session: 1
timestamp: 2026-03-03T01:45:00Z
orientation:
  current: "Diagnosed: missing mobile back navigation in ToolRunPanel. Fix is clear, minimal, and follows the established pattern used in five other pages."
  next: "Create a fix feature or implement directly — add ArrowLeft back button in ToolRunPanel header, md:hidden."
  commit: "Diagnosis complete. Exact code changes identified. Ready to implement."
---

**jordan · Owner**
The tools page layout doesn't work for mobile

---

## Session 1 — Diagnosis and Fix Design

### Loading context

This is a fresh ponder — no prior sessions. I read the full `ToolsPage.tsx` and compared it to `PonderPage.tsx`, `InvestigationPage.tsx`, `EvolvePage.tsx`, `GuidelinePage.tsx`, and `KnowledgePage.tsx`.

---

### What's actually broken

**Ben Hartley · Developer Productivity UX:**
> Let me be precise here. Jordan says "doesn't work for mobile" — that's a feeling, not a spec. Let's find the exact friction points before jumping to solutions.

After reading the code:

**Issue 1: No back navigation (critical)**

`ToolsPage` handles the show/hide correctly:
- List panel: `selectedTool ? 'hidden md:flex' : 'flex'` — hides on mobile when a tool is selected ✓
- Detail panel: `!selectedTool ? 'hidden md:flex items-center justify-center' : 'flex flex-col'` — shows on mobile when a tool is selected ✓

But when a user taps a tool on mobile, they're taken to the detail view with **no path back to the list**. The only escape is:
- Reload the page
- Tap the Sidebar nav item (if it's accessible)

This is the critical failure. You're functionally trapped.

**Ben Hartley:**
> This is exactly the "dead-end navigation" anti-pattern. On mobile, every drill-down needs an escape hatch. Without it, users learn to avoid the page entirely — especially if they realize they picked the wrong tool.

**Issue 2: ToolRunPanel has no mobile header affordance**

`ToolRunPanel` renders a header at line 887 with the tool name and description, but no back button. Compare every other page in this codebase that has a master/detail pattern:

| Page | Back button? | Pattern |
|---|---|---|
| PonderPage | ✓ | `showMobileDetail ? 'hidden md:flex' : 'flex'` + ArrowLeft |
| InvestigationPage | ✓ | Same |
| EvolvePage | ✓ | Same |
| GuidelinePage | ✓ | Same |
| KnowledgePage | ✓ | Same |
| **ToolsPage** | **✗** | **MISSING** |

---

### What's already correct

**Ben Hartley:**
> Credit where it's due — the show/hide logic is implemented. This isn't a "page was never mobile-considered" situation, it's an incomplete implementation. One missing affordance.

- `flex-wrap` on action buttons at line 1026 ✓
- `overflow-y-auto` on the main panel ✓
- Tab row at line 954 uses compact spacing ✓
- History records truncate with `truncate` class ✓

The interior layout of `ToolRunPanel` is fine on narrow viewports. The entire problem is: **no back button**.

---

### Comparing to the broken state in ThreadsPage

Quick aside: `ThreadsPage` at line 92 has a similar non-responsive sidebar — `w-[280px] shrink-0 border-r border-border flex flex-col overflow-hidden md:flex md:w-[280px]`. The `md:` prefix doesn't hide on mobile, it's just a width override. That's a separate (worse) problem. Not in scope here.

---

### Fix design

⚑  **Decided:** Minimal targeted fix — add mobile back navigation to ToolRunPanel. Match the established pattern exactly.

**Change 1:** Add `ArrowLeft` to lucide-react imports in ToolsPage (line 8)

**Change 2:** Add `onBack?: () => void` to `ToolRunPanelProps` (line 732–735)

**Change 3:** Add back button in `ToolRunPanel` header, `md:hidden`, before the tool name/description row (line ~887–888)

**Change 4:** Pass `onBack={() => setSelectedName(null)}` to `ToolRunPanel` in ToolsPage (line 1297)

**Ben Hartley:**
> Keep it `md:hidden`. We don't want the back button showing on desktop — the sidebar is always visible there. The button only makes sense on mobile where the list is hidden. This is exactly what the other pages do.

?  **Open:** Should the button say "Tools" or just be a bare arrow? PonderPage uses a bare arrow without label. InvestigationPage uses `ArrowLeft` + "Back". KnowledgePage uses `ArrowLeft` + "Back". Most common pattern in this app is unlabeled arrow. Either works — lean toward matching the most common pattern.

⚑  **Decided:** Use bare `ArrowLeft` with `p-1.5` — matches the button style used elsewhere in the header area. Add `title="Back to tools"` for accessibility.

---

### Estimated impact

- 1 file changed
- ~12 lines total (import + prop + button + usage)
- No new components
- No API changes
- No state changes — `selectedName` state already handles the toggle; setting it to `null` reveals the list

---

### Commit signal

This ponder is converging. The problem is diagnosed, the fix is identified, the pattern to follow is clear, the scope is tiny. Ready to implement directly.

**Next:** Create a fix feature (`fix-toolspage-mobile-back-nav`) or implement inline in the next `/sdlc-next` cycle. The fix is a direct, non-architectural change — implement directly is preferred.
