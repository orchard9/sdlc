---
session: 1
timestamp: 2026-03-07T16:45:00Z
orientation:
  current: "Problem is scoped and solution is designed — released milestones show wrong UI (Run UAT button), need a ReleasedPanel component"
  next: "Commit to milestones/features — one bug fix (hide Run UAT for released), one small feature (ReleasedPanel with victory banner + next milestone link)"
  commit: "This is ready to commit — the solution is small, well-scoped, and clearly needed"
---

**Xist · Owner**
milestone status after uat success

http://localhost:7777/milestones/v48-milestone-feature-forward-motion

this milestone has passed UAT and claims to be released, but there is still a "Run UAT" button, and other than a "released" tag, there is no indication of what to do next.

what are the next steps for a milestone at this point? iterate on the ui to give the user buttons to perform next steps.

---

## Investigation

Loaded the milestone detail page for `v48-milestone-feature-forward-motion`. The API confirms status is `released` with `released_at: 2026-03-07T16:33:08`. The page itself was stuck loading in Playwright, but the underlying APIs work correctly.

### The Bug

In `MilestonePreparePanel.tsx`, the `VerifyingMini` component renders when:
```js
const isVerifying = waves.length === 0
  && milestone_progress.total > 0
  && milestone_progress.released === milestone_progress.total
```

This condition is true for **both** verifying AND released milestones — it only checks that all features are released, not whether `released_at` is set. So released milestones incorrectly show the "Run UAT" button and "All features released" banner.

### The Deeper Problem

After a milestone reaches `released`, the UI offers zero affordances. Just a badge. The user's question — "what do I do now?" — has no answer on the page.

### Data Discovery

- 48 milestones total, most released
- 30+ stuck in `verifying` (all features released but UAT never run or not passed)
- `MilestoneStatus` enum: `Active | Verifying | Released | Skipped`
- `released_at` timestamp available on the API response
- UAT runs with verdicts, screenshots, test counts available per milestone

---

## Ben Hartley (Dev Productivity UX)

The released state should feel like a **landing page**, not a dead end. The victory banner is the hero. Action buttons (re-run UAT, view report) are secondary. A "next milestone" link provides navigation forward.

Key principle: **"what do I do now?" must be answerable without scrolling.** The released milestone detail page should communicate completion at a glance.

Don't add a "Tag Release" button that mixes git operations into the UI. If needed, show a copyable command block — that's the established pattern. But honestly, not every milestone maps 1:1 to a git tag.

## Dana Cho (Product Skeptic)

Who visits a released milestone page? Two personas:

1. **The owner checking status** — wants confirmation it shipped. Victory banner = done.
2. **A teammate discovering context** — wants to understand what shipped. Feature list + UAT history = done.

The "next steps" framing over-indexes on forward motion. The core problem is simpler: (a) wrong button shown, (b) no visual signal of completion beyond a tiny badge. Fix those two things. Don't over-build.

The "next milestone" hint is nice if cheap, but it's not the core value.

## Resolution

Dana's scope discipline is right. Ben's "landing page" framing is right.

### Decisions

⚑ **Decided:** Fix the bug — `MilestonePreparePanel` must check milestone status and hide `VerifyingMini` when released.

⚑ **Decided:** Add a `ReleasedPanel` component that replaces `VerifyingMini` for released milestones:
- Victory banner (emerald) with released_at date, feature count, latest UAT verdict + stats
- Small "Re-run UAT" secondary button for regression testing
- "Next milestone" link if an active/verifying milestone exists

⚑ **Decided:** The panel needs `milestone.status` and `milestone.released_at` — either pass as props from `MilestoneDetail` parent, or include status in the prepare API response.

? **Open:** Whether to show the `git tag` release command. Leaning no — release process is documented in CLAUDE.md and not every milestone = a version tag.

### Implementation Shape

1. **Bug fix** — In `MilestonePreparePanel`, add status awareness. When status is `released`, render `ReleasedPanel` instead of `VerifyingMini`.
2. **ReleasedPanel component** — New component in `frontend/src/components/milestones/`:
   - Fetches latest UAT run from history
   - Shows released_at, feature count, UAT pass stats
   - Secondary "Re-run UAT" button
   - Optional "Next milestone →" link (query milestones API for first active/verifying)
3. **Props threading** — `MilestoneDetail` already has the milestone object. Pass `status` and `released_at` down to `MilestonePreparePanel`.

This is 2-3 files touched, ~100 lines of new code. Clean, small scope.
