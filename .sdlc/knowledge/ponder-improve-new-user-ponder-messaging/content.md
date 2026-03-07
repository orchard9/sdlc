---
session: 1
timestamp: 2026-03-07T17:40:00Z
orientation:
  current: "Implementation complete — rich onboarding content replaces blank empty state on /ponder"
  next: "Verify on mobile viewport, consider similar treatment for other workspace pages (Root Cause, Evolve, Guidelines, Spikes)"
  commit: "Owner confirms the messaging and layout work for new users"
---

**Xist · Owner**
improve new user ponder messaging

http://localhost:7777/ponder

When the user is viewing this url there is a large essentially blank space where the featured content should be.  There are a few buttons, but a LOT of blank space.

Iterate on this page and add a big picture overview of "What is Ponder?" and "How do I Ponder" focussed specifically on ponder in the context of this page.

---

## Session 1: Fill the blank canvas

### Problem

The `/ponder` page detail pane (right side) shows almost nothing when no idea is selected — just a small lightbulb icon, "Select an idea to explore", and two tiny buttons. On a 1440px+ monitor this is a massive empty void that tells a new user nothing about what Ponder is or how to use it.

### Analysis

Examined `frontend/src/pages/PonderPage.tsx` lines 917-939 — the empty state was only 23 lines of JSX. Compared with the dashboard empty state (`DashboardEmptyState.tsx`) which has a richer approach with suggestion chips and context-aware prompts.

The ponder page is the entry point for ideation. A new user landing here needs to understand:
1. What Ponder is (not a task tracker — it's a thinking space)
2. The workflow (seed → explore → capture → commit)
3. The lifecycle (exploring → converging → committed)
4. What to do next (suggest or create)

### Implementation

Replaced the minimal empty state with a structured onboarding layout:

- **Hero**: "Think before you build." — a tagline that positions Ponder as intentional thinking, not busywork. Backed by a one-sentence value prop.
- **How it works (4 steps)**: Numbered cards with titles and descriptions. Walks the user through the full Ponder workflow from rough idea to committed milestones.
- **Lifecycle strip**: Visual pill badges (Exploring → Converging → Committed) with color-coded borders matching the status badge colors used elsewhere in the app. Brief paragraph explaining each stage.
- **CTAs**: Two prominent buttons — "Suggest an idea" (primary fill) and "New idea" (outlined). Larger and more visible than the original tiny text links.
- **Contextual hint**: "Or select an idea from the list to continue exploring" — only shown when there are existing entries, to gently redirect returning users.

### Design decisions

- **Scrollable container** (`h-full overflow-y-auto`) — works at any viewport height without clipping
- **Max-width constraint** (`max-w-xl`) — keeps line lengths readable, centered in the detail pane
- **No new files** — all content is inline in the PonderPage detailPane ternary. No new components or abstractions for what is a single-use onboarding block.
- **Consistent styling** — uses existing Tailwind classes and color tokens from the design system (primary, muted-foreground, border, card)

### Verification

- Built frontend (`npm run build`)
- Rebuilt Rust binary (`cargo install --path crates/sdlc-cli`)
- Restarted server
- Verified via Playwright screenshot — content renders correctly, fills the space, is readable and well-structured

### Artifacts

- `implementation.md` — full change description captured to scrapbook

### Open questions

? Open: Should other workspace pages (Root Cause, Evolve, Guidelines, Spikes, Knowledge) get similar treatment? They likely have the same blank-space problem.

? Open: Mobile viewport not yet tested — should verify the 4-step cards don't overflow on narrow screens.
