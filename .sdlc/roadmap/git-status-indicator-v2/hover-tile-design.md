# Design: Git Details Hover Tile

## Problem
The git status chip shows a severity dot and one-line summary. The API already returns rich data (dirty files, staged files, untracked count, ahead/behind, conflicts) but it's invisible to the user. Users need to understand what's happening and what action to take.

## Proposal: Hover/Touch Detail Panel

### Trigger
- **Desktop:** mouse enter on the git status chip area (not just the dot)
- **Mobile:** tap on the chip (toggle open/close)
- **Dismiss:** mouse leave (desktop), tap outside or tap again (mobile)

### Content (structured, not a wall of text)
```
┌─────────────────────────────┐
│  main                    ●  │  ← branch name + severity dot
├─────────────────────────────┤
│  6 modified files            │
│  2 staged for commit         │
│  3 untracked files           │
│  0 ahead · 0 behind         │
├─────────────────────────────┤
│  ▸ Ready to commit (2 staged)│  ← actionable guidance
│  ▸ Stage your changes first  │
│  [Commit]                    │  ← commit button lives here
└─────────────────────────────┘
```

### Guidance logic (frontend-only, derived from API fields)
- `has_conflicts` → "Resolve merge conflicts before proceeding" (red)
- `behind > 0` → "Pull upstream changes ({N} commits behind)"
- `staged_count > 0` → "Ready to commit ({N} staged)" + show Commit button
- `dirty_count > 0 && staged_count == 0` → "Stage your changes to commit"
- `untracked_count > 5` → "Consider adding a .gitignore ({N} untracked)"
- all clean → show the GitGreenQuote

### Positioning
- Pops up above/beside the chip (bottom of sidebar)
- Uses Radix Popover or a simple absolute-positioned div
- Matches existing card styling (bg-card, border-border, shadow)

## ⚑ Decided: hover tile is the right UX
The chip is too small for detail. A full page is too heavy. A hover tile bridges the gap — glanceable detail without navigation.

## ? Open: should the tile show file names?
Listing all dirty/staged files could be noisy. Options:
1. Just counts (clean, fast)
2. First 5 files + "and N more" (useful)
3. Full list (overwhelming)
Leaning toward option 2 — but this needs the API to return file lists, which it currently doesn't.