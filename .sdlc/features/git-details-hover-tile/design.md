# Design: Git Details Hover Tile

## Component Architecture

```
GitStatusChip (existing)
  └── GitDetailsPopover (new)
        ├── BranchSection
        ├── StatusCountsSection
        ├── SeverityBadge
        └── GuidanceLine
```

## Component: GitDetailsPopover

**File:** `frontend/src/components/layout/GitDetailsPopover.tsx`

### Props

```typescript
interface GitDetailsPopoverProps {
  status: GitStatus
  anchor: 'above' | 'right'  // position relative to trigger
}
```

### Layout (vertical stack)

```
┌─────────────────────────────────┐
│  main  ·  2 ahead              │  ← branch + tracking
├─────────────────────────────────┤
│  ● 3 modified                  │  ← dirty_count (yellow dot)
│  ● 2 staged                    │  ← staged_count (green dot)
│  ● 1 untracked                 │  ← untracked_count (gray dot)
├─────────────────────────────────┤
│  ▲ Yellow — uncommitted changes│  ← severity with explanation
├─────────────────────────────────┤
│  Stage and commit when ready.  │  ← guidance text
└─────────────────────────────────┘
```

- Each status row only renders when count > 0
- Minimum content: branch line + severity + guidance (always shown)
- Width: 240px fixed, max-height scrollable if needed

### Severity Explanation Map

| Severity | Condition | Explanation |
|----------|-----------|-------------|
| green | clean | "Working tree clean" |
| green | ahead > 0 | "Ahead of upstream" |
| yellow | dirty > 0 | "Uncommitted changes" |
| yellow | behind > 0 | "Behind upstream" |
| yellow | untracked > 5 | "Many untracked files" |
| red | conflicts | "Merge conflicts" |
| red | behind > 10 | "Far behind upstream" |

### Guidance Map

| Severity | Guidance |
|----------|----------|
| green (clean, ahead=0) | "All clear. Nothing to commit." |
| green (ahead > 0) | "Push to share your commits." |
| yellow | "Uncommitted changes. Stage and commit when ready." |
| red (conflicts) | "Resolve conflicts before continuing." |
| red (behind) | "Pull from upstream to catch up." |

## Integration with GitStatusChip

The `GitStatusChip` component gains:

1. A `useState<boolean>` for popover visibility (`open`)
2. `onMouseEnter` / `onMouseLeave` handlers with a 150ms debounce on leave
3. An `onClick` handler that toggles `open`
4. A click-outside listener (via `useEffect` + `ref`) to dismiss
5. Conditional render of `<GitDetailsPopover>` when `open && status`

### Positioning Strategy

CSS-only positioning using `position: relative` on the chip wrapper and `position: absolute` on the popover:

- **Expanded sidebar**: popover appears to the right of the chip, aligned to bottom edge
- **Collapsed sidebar**: popover appears to the right (since chip is icon-only, popover extends into content area)

A `z-50` ensures the popover floats above sidebar and content.

## GitStatus Interface Update

In `frontend/src/hooks/useGitStatus.ts`, add the three missing fields:

```typescript
export interface GitStatus {
  branch: string
  dirty_count: number
  staged_count: number
  untracked_count: number    // NEW
  ahead: number
  behind: number
  has_conflicts: boolean
  conflict_count: number     // NEW
  severity: 'green' | 'yellow' | 'red'
  summary: string            // NEW
}
```

These fields are already returned by the Rust API (`git.rs` `GitStatus` struct) but were omitted from the TypeScript interface.

## Styling

- Background: `bg-popover` (card-like dark surface)
- Border: `border border-border rounded-lg shadow-lg`
- Text: `text-sm text-popover-foreground`
- Section dividers: `border-t border-border/50`
- Status dots: same color classes as the chip severity dot
- Padding: `p-3` with `space-y-2` between sections
- Animation: `animate-in fade-in-0 zoom-in-95` (Tailwind animate plugin, or simple opacity transition)

## Files Changed

| File | Change |
|------|--------|
| `frontend/src/components/layout/GitDetailsPopover.tsx` | New component |
| `frontend/src/components/layout/GitStatusChip.tsx` | Add hover/click trigger, render popover |
| `frontend/src/hooks/useGitStatus.ts` | Add `untracked_count`, `conflict_count`, `summary` to interface |

## Mockup

[Mockup](mockup.html)
