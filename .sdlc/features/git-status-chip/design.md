# Design: Git Status Chip

## Component Architecture

### GitStatusChip

A single React component placed in the Sidebar's bottom utility section. It manages its own data fetching via a custom hook.

```
Sidebar.tsx
  └── bottom utility area
       ├── Ask Code button
       ├── Fix Right Away button
       ├── Search button
       └── GitStatusChip  <-- new
```

### File Structure

```
frontend/src/components/layout/
  └── GitStatusChip.tsx    — component + hook
```

### Custom Hook: useGitStatus

```typescript
interface GitStatus {
  branch: string;
  dirty_count: number;
  staged_count: number;
  ahead: number;
  behind: number;
  has_conflicts: boolean;
  severity: 'green' | 'yellow' | 'red';
}

function useGitStatus(intervalMs: number = 10000): {
  status: GitStatus | null;
  loading: boolean;
  error: boolean;
}
```

- Polls `GET /api/git/status` every `intervalMs`.
- Pauses polling when `document.hidden === true` (listens to `visibilitychange`).
- Re-fetches immediately on window focus.
- Returns `{ status, loading, error }`.

### Severity Mapping

| API severity | Dot color | Tailwind class |
|---|---|---|
| `green` | Green | `bg-emerald-500` |
| `yellow` | Yellow/Amber | `bg-amber-500` |
| `red` | Red | `bg-red-500` |
| Error/loading | Grey | `bg-muted-foreground/30` |

### Summary Text Logic

```
if has_conflicts   → "{branch} - {n} conflicts"
if dirty_count > 0 → "{branch} - {dirty_count} modified"
if ahead > 0       → "{branch} - {ahead} ahead"
else               → "{branch} - clean"
```

### Collapsed vs Expanded

- **Expanded sidebar**: Renders like other utility buttons — icon (colored dot) + text + optional commit badge.
- **Collapsed sidebar**: Icon-only (colored dot), with `title` tooltip showing full summary text.

### Commit Button

- Rendered as a small badge/button next to the summary text when `staged_count > 0`.
- Shows a git-commit icon from lucide-react.
- On click: `POST /api/git/commit` with default message (or triggers a commit dialog — initial implementation uses a simple POST).
- Hidden when no staged files.

### Integration with Sidebar

The `GitStatusChip` receives the `collapsed` prop from Sidebar. It is added as a new element in the bottom utility `<div>`, positioned as the first item (above Ask Code) so git status is the most prominent utility.

### Props

```typescript
interface GitStatusChipProps {
  collapsed: boolean;
}
```

## Data Flow

```
GitStatusChip mounts
  → useGitStatus hook starts polling GET /api/git/status
  → API returns GitStatus JSON
  → Component renders severity dot + summary text
  → If staged_count > 0, show commit button
  → On commit click → POST /api/git/commit
  → Re-fetch status after commit
```

## Error Handling

- Network error or non-200 response: set `error: true`, render grey dot with "Git status unavailable" tooltip.
- API returns unexpected shape: treat as error state.
- No retry backoff needed — the regular polling interval handles recovery.

## Mockup

[Mockup](mockup.html)
