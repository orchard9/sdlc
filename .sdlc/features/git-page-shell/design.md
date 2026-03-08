# Design: Git Page Shell

## Component Architecture

```
App.tsx
  Route /git       → GitPage
  Route /git/*     → GitPage

GitPage.tsx
  ├── WorkspaceShell
  │   ├── listPane  → GitListPane (branch header + placeholder file list)
  │   └── detailPane → GitDetailPane (placeholder detail view)
  └── URL-driven showDetail (useParams for wildcard path)
```

## Sidebar Change

In `Sidebar.tsx`, add to the `integrate` group after "Network":

```typescript
{ path: '/git', label: 'Git', icon: GitBranch, exact: false },
```

`GitBranch` is already imported in Sidebar.tsx.

## GitPage Component

### File: `frontend/src/pages/GitPage.tsx`

**State:**
- `selectedPath: string | null` — derived from URL wildcard param `*`.
- Git status from `useGitStatus()` hook (already exists).

**Layout:**
- Uses `WorkspaceShell` with `showDetail = !!selectedPath`.

**Left pane (GitListPane):**
- Header row: branch name + severity dot (from `useGitStatus`), status summary text.
- Below header: placeholder text "File browser will appear here" styled as empty state.
- Future: populated by `git-file-browser-ui` feature.

**Right pane (GitDetailPane):**
- When no file selected: centered empty state with `Files` icon + "Select a file to view details".
- When file selected: placeholder showing the selected path. Future: populated by diff viewer.

**Mobile back navigation:**
- When `showDetail` is true on mobile, show a back button that navigates to `/git`.

## Route Registration

In `App.tsx`:
```tsx
import GitPage from './pages/GitPage'  // or lazy import

<Route path="/git" element={<GitPage />} />
<Route path="/git/*" element={<GitPage />} />
```

## Visual Design

- Matches existing workspace pages (Investigations, Evolve, etc.).
- Left pane width: default `w-72` from WorkspaceShell.
- Branch header uses the same severity dot pattern as GitStatusChip.
- Empty states use muted text + icon pattern consistent with other pages.

## Mockup

[Mockup](mockup.html)
