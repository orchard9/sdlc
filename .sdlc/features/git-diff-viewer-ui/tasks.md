# Tasks: git-diff-viewer-ui

## Task 1: Install @git-diff-view/react dependency
Add `@git-diff-view/react` and `@git-diff-view/core` as production dependencies in `frontend/package.json`. Run `npm install` to verify clean resolution.

## Task 2: Implement DiffViewer component with useDiff hook
Create `frontend/src/components/DiffViewer.tsx` containing:
- `useDiff` hook that fetches from `/api/git/diff?path=<filePath>&old=<oldRef>&new=<newRef>`
- `DiffViewer` component that renders the diff using `@git-diff-view/react`
- Internal sub-components: DiffHeader (file path, stats, view toggle), loading skeleton, error state, empty state, binary file state
- Props: `filePath`, `oldRef?`, `newRef?`, `defaultView?`

## Task 3: Apply blue/amber colorblind-safe theme
Create `frontend/src/components/DiffViewer.css` with CSS overrides for `@git-diff-view/react` default styling:
- Additions: blue-500 tinted backgrounds and borders
- Deletions: amber-500 tinted backgrounds and borders
- Gutter and line numbers using muted design tokens
- Dark theme integration matching existing card/muted backgrounds

## Task 4: Add unified/split view toggle with responsive collapse
Implement the view mode toggle in DiffHeader:
- Segmented control switching between unified and split
- Auto-collapse to unified below 1024px viewport width
- Wrap toggle for long lines
