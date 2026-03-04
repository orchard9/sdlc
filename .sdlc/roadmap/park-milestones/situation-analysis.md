# Situation Analysis: Milestone Parking

## Current State
- 44 total milestones
- 27 released, 12 verifying, 5 active, 0 skipped
- `MilestoneStatus` enum: `Active | Verifying | Released | Skipped`
- `Skipped` semantics: **cancelled/intentionally bypassed** (`skipped_at` timestamp)
- No concept of 'paused' or 'parked' — a milestone is either in-flight or permanently done

## The Gap
12 milestones stuck in 'verifying' — code-complete features but no UAT sign-off. Some of these (citadel-*, telegram-digest-bot) are aspirational/future work that shouldn't be claiming active screen real estate.

5 active milestones, but not all are being worked on right now. Some were committed too early (ponder-commit'd before the owner was ready to prioritize them).

## UI Impact
- `MilestonesPage.tsx` splits into 'active' (everything non-released) and 'archive' (released only)
- `HorizonZone.tsx` shows milestones where all features are still in draft — polluted by parked work
- Dashboard noise: 17 milestones (5 active + 12 verifying) show in the non-archive section
- No way to say 'not now' without saying 'never' (skip/cancel)

## CLI Surface
- `sdlc milestone skip` / `sdlc milestone cancel` — permanent cancellation
- No `sdlc milestone park` or equivalent 'pause' command
