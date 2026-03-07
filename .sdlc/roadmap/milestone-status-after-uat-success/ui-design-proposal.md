## UI Design Proposal: Released Milestone State

### Immediate Fix (Bug)
In `MilestonePreparePanel.tsx`, the `isVerifying` check must also verify the milestone is NOT released. The panel needs access to `milestone.status` — either passed as a prop or fetched from the milestone API alongside prepare data.

### Released Milestone Panel (replaces VerifyingMini)

When `status === "released"`, show a **ReleasedPanel** instead:

```
┌──────────────────────────────────────────────────┐
│ ✓ Released · Mar 7, 2026                         │
│                                                   │
│   3/3 features · UAT passed · 54 agent turns      │
│                                                   │
│   [Tag Release]  [Re-run UAT]  [View UAT Report]  │
│                                                   │
│   Next milestone: v49-xxx (active, 2/5 features)  │
│   → View milestone                                │
└──────────────────────────────────────────────────┘
```

### Components
1. **Victory banner** — emerald, shows released_at, feature count, UAT verdict
2. **Action buttons**:
   - **Tag Release** — copies `git tag vN && git push origin vN` to clipboard (the sdlc release process)
   - **Re-run UAT** — for regression testing (keep the button, but secondary/muted)
   - **View UAT Report** — scrolls to UAT history or opens latest run
3. **Next milestone hint** — shows the next active milestone with progress, links to it

### What NOT to Build
- No "archive" button — milestones dont need archival, they just are
- No "reopen" — released is terminal (can only skip)
- No changelog generation — thats a release process concern, not a milestone UI concern

### Verifying Milestones (Separate Concern)
30+ milestones stuck in `verifying` state. The verifying panel is correct for those — "Run UAT" makes sense. The only bug is that released milestones also show it.
