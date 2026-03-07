## Current State Analysis

### The Bug
The `VerifyingMini` component in `MilestonePreparePanel.tsx` checks:
```js
const isVerifying = waves.length === 0 && milestone_progress.released === milestone_progress.total
```
This is true for **both** verifying AND released milestones. A released milestone still shows "Run UAT" because nothing distinguishes these states in the prepare panel.

### The Deeper Problem
Once a milestone reaches `released`, the UI has **no affordances** — just a badge. The user sees:
1. A "released" status badge
2. A "Run UAT" button (wrong — should not appear)
3. Feature list (all released)
4. UAT history
5. **Nothing about what comes next**

### What "Next" Means for Released Milestones
The milestone lifecycle is: `Active → Verifying → Released`. After released:
- **Tag a release** — `git tag vX.Y.Z && git push origin vX.Y.Z` (the release process)
- **Start next milestone** — which active milestone is queued?
- **Archive** — collapse into a summary for the timeline
- **Celebrate** — show the UAT pass with its stats as a victory state

### Data Available
- `released_at` timestamp
- UAT run history with verdicts, screenshots, durations
- Feature list with all phases
- Milestone title/description/vision
