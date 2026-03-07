# Spec: Rich Onboarding Empty States for All Workspace Pages

## Problem

When no item is selected in a workspace page (Ponder, Root Cause, Guidelines, Spikes, Knowledge), the detail pane shows a minimal placeholder — a faded icon and one-line hint. New users get no context about what the workspace does, how it works, or what action to take next. This creates a cold-start problem where the UI fails to teach the product.

## Solution

Replace each workspace page's "no selection" detail pane with a structured onboarding empty state containing three sections:

1. **Hero** — workspace icon (in a tinted container), one-line tagline, and a short paragraph explaining what the workspace is for.
2. **How it works** — a vertical list of 3-4 steps/concepts, each with an icon, title, and one-line description. The content is specific to each workspace (not generic).
3. **CTA** — a primary action button that opens the workspace's "new item" flow (New Guideline, New Root Cause, Suggest an Idea, etc.). When items already exist, a secondary hint says "Or select an item from the list."

## Affected Pages

| Page | File | Hero tagline | CTA |
|------|------|--------------|-----|
| Ponder | `PonderPage.tsx` | "Think before you build." | Suggest an idea / New idea |
| Root Cause (Investigation) | `InvestigationPage.tsx` | "Find the root cause." | New Root Cause |
| Guidelines | `GuidelinePage.tsx` | "Codify what works." | New Guideline |
| Spikes | `SpikePage.tsx` | "Answer one question fast." | `/sdlc-spike` command |
| Knowledge | `KnowledgePage.tsx` | "What the team knows." | `sdlc knowledge add` command |

## Design Constraints

- All empty states follow the same visual pattern: centered `max-w-xl` container, `py-10 px-6`, `space-y-8` between sections.
- Icons use `lucide-react` from existing imports — no new icon library.
- Step items use `border border-border/50 bg-card/50` card style with icon + title + description.
- CTA buttons use the existing `bg-primary text-primary-foreground` button style.
- Spikes page additionally shows a "Verdicts" strip explaining ADOPT/ADAPT/REJECT.
- Knowledge page additionally shows lifecycle info about catalog organization and staleness tracking.
- Ponder page additionally shows a "Lifecycle" strip (Exploring -> Converging -> Committed).

## Out of Scope

- The Hub page is not a workspace page in the same sense; it has its own empty state patterns for fleet-level content.
- No changes to list pane empty states (those already exist with short messages and "+ New" buttons).
- No new components — the empty states are inline JSX in each page component.

## Acceptance Criteria

- [ ] Every workspace page shows the structured Hero / How-it-works / CTA when no item is selected
- [ ] CTA buttons trigger the correct "new item" flow for each workspace
- [ ] When items exist, a "select from list" hint appears below the CTA
- [ ] All empty states are visually consistent (same spacing, typography, card style)
- [ ] No regressions in the selected-item detail pane view
