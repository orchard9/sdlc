---
session: 1
timestamp: 2026-03-07T17:40:00Z
orientation:
  current: "Implemented rich empty-state content for the Evolve page detail pane"
  next: "Verify visually in browser, iterate on copy/layout if needed"
  commit: "Owner confirms the page no longer feels empty and communicates Evolve's purpose clearly"
---

**Xist · Owner**
improve-new-user-evolve-messaging

http://localhost:7777/evolve

When the user is viewing this url there is a large essentially blank space where the featured content should be.  There are a few buttons, but a LOT of blank space.

Iterate on this page and add a big picture overview of "What is Evolve?" and "How do I Evolve" focussed specifically on evolve in the context of this page.

---

## Session 1 — Fill the blank space with onboarding content

### Problem diagnosis

Navigated to the EvolvePage component (`frontend/src/pages/EvolvePage.tsx`). The page uses a `WorkspaceShell` two-pane layout:

- **Left pane**: List of evolve sessions with status filter tabs
- **Right pane (detail)**: When no session is selected, renders only a wrench icon, "Select a session to continue", and a keyboard shortcut hint — a ~20px cluster in the center of a full-height pane. The rest is blank.

### What Evolve actually is (from codebase)

Evolve is an agent-driven investigation workspace with a 5-phase flow:

1. **Survey** — scan system structure, entry points, docs state, TODOs/FIXMEs
2. **Analyze** — score maturity across 5 lenses (pit of success, coupling, growth readiness, self-documenting, failure modes)
3. **Paths** — propose 2-4 evolution paths with effort/impact tradeoffs
4. **Roadmap** — build phased roadmap (proper solution → enabling changes → extended vision)
5. **Output** — produce action plan with chosen path, rationale, and concrete next steps

The output can be committed into milestones and features via `/sdlc-ponder-commit`.

### Implementation

Replaced the sparse empty detail pane with a rich onboarding section containing:

1. **"What is Evolve?"** — one-paragraph explanation of purpose and output
2. **"How does it work?"** — 5-phase visual flow with icons (Search, BarChart3, GitFork, Map, Zap) and one-line descriptions per phase
3. **CTA button** — "Start an Evolution" that opens the create modal
4. **Subtext** — "Describe the pain point or area you want to improve and an agent takes it from there."

Follows the same visual patterns as `DashboardEmptyState` and `SpikeEmptyState` — card-based layout, muted icons, concise copy.

Removed unused `Wrench` import; added `Search, BarChart3, GitFork, Map, Zap` from lucide-react.

### Decisions

- ⚑ Decided: Content lives in the detail pane (right side), not the list pane — the list pane already has its own empty state with a "New Evolution" button
- ⚑ Decided: Phase descriptions are single sentences, not paragraphs — the page should orient, not overwhelm
- ⚑ Decided: Used existing lucide-react icons rather than custom SVGs — consistent with the rest of the UI
- ? Open: Whether the list pane empty state also needs enrichment (currently shows "No evolution sessions yet." which is sparse but functional)
