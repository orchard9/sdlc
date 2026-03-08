# Design: History Tab UI with Compact Commit List

## Component Architecture

```
App.tsx
  Route /git  --> GitPage (new page)
    TabBar: [History | Files | Diff]   (only History active initially)
    GitHistoryTab
      CommitRow (repeated)
      LoadMoreButton
```

## New Files

| File | Purpose |
|------|---------|
| `frontend/src/pages/GitPage.tsx` | Page shell with tab bar, renders active tab |
| `frontend/src/components/git/GitHistoryTab.tsx` | Commit list with pagination |
| `frontend/src/hooks/useGitLog.ts` | Data-fetching hook for `GET /api/git/log` |

## Modified Files

| File | Change |
|------|--------|
| `frontend/src/App.tsx` | Add `/git` route |
| `frontend/src/components/layout/Sidebar.tsx` | Add "Git" nav entry under "integrate" group |

## Data Types

```typescript
interface GitCommit {
  hash: string        // full 40-char SHA
  short_hash: string  // 7-char abbreviated SHA
  message: string     // first line of commit message
  author: string      // author name
  timestamp: string   // ISO 8601 timestamp
}

interface GitLogResponse {
  commits: GitCommit[]
  total: number       // total commit count (for pagination)
  has_more: boolean   // whether more commits exist
}
```

## Hook: `useGitLog`

```typescript
function useGitLog(limit = 50) {
  // Returns { commits, loading, error, hasMore, loadMore }
  // - Initial fetch: GET /api/git/log?limit=50
  // - loadMore: GET /api/git/log?limit=50&offset=<current_count>
  // - Appends new commits to existing array
  // - Handles 404 (API not available) gracefully
}
```

## Layout Design

### CommitRow

A single-line horizontal layout:

```
[short_hash]  commit message (truncated)                    author    2h ago
```

- `short_hash`: monospace, `text-muted-foreground`, clickable (future: links to diff)
- `message`: `text-sm`, `truncate` class, `flex-1` to fill available space
- `author`: `text-xs`, `text-muted-foreground`
- `timestamp`: `text-xs`, `text-muted-foreground`, right-aligned

### Tab Bar

Horizontal tab bar at top of GitPage, using simple underline-active styling consistent with existing Ponder UI patterns. Initially only "History" tab is active; "Files" and "Diff" tabs are placeholders added by sibling features.

### Responsive Behavior

- On narrow screens, author and timestamp stack below the message
- Hash always visible
- Message truncation adjusts to available width

## Relative Time Formatting

Use a lightweight utility function (no external dependency):

```typescript
function relativeTime(isoString: string): string {
  // Returns "just now", "5 min ago", "2 hours ago", "3 days ago", etc.
}
```

## Error States

| State | UI |
|-------|-----|
| Loading | Skeleton rows (6 rows, pulse animation) |
| Empty | Centered "No commits yet" with muted icon |
| API 404 | "Commit history not available" (API feature not deployed) |
| API error | "Failed to load" with retry button |
| Not a git repo | "Not a git repository" centered message |

## Mockup

[Mockup](mockup.html)
