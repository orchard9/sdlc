# Tasks: Git Details Hover Tile

## Task 1: Update GitStatus TypeScript interface
Add `untracked_count`, `conflict_count`, and `summary` fields to the `GitStatus` interface in `frontend/src/hooks/useGitStatus.ts` to match the Rust API response.

## Task 2: Create GitDetailsPopover component
Build `frontend/src/components/layout/GitDetailsPopover.tsx` with branch/tracking section, status count rows (dirty, staged, untracked, conflicts -- zero counts omitted), severity badge with explanation, and guidance line. Use existing Tailwind classes consistent with sidebar styling.

## Task 3: Integrate popover into GitStatusChip
Add hover/click trigger to `GitStatusChip` that shows/hides `GitDetailsPopover`. Include 150ms leave delay, click toggle, click-outside dismiss, and proper positioning for both expanded and collapsed sidebar states.
