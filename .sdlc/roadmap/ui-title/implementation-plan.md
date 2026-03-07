## Implementation Plan

### Current State
- `index.html` has static `<title>sdlc</title>`
- `AppShell` already has `projectName` (from config API) and `titleFromPath(pathname)` which maps routes to labels
- Detail routes (`/ponder/:slug`, `/features/:slug`, etc.) have the slug in the URL

### Target Format
```
PROJECT · FOCUS CONTENT · Ponder
```

**Decision: Use `·` (middle dot) instead of `$` as separator.** The `$` character reads as a shell variable or currency symbol. The middle dot is the standard separator used by GitHub, Google, Slack, and most modern web apps in browser titles. It is visually clean and unambiguous.

### Architecture

**Option A — `useEffect` in AppShell (minimal)**
Add a single `useEffect` in `AppShell` that sets `document.title` based on `projectName` and `titleFromPath(pathname)`. For detail views, extract the slug from the URL and use it as the content name.

- Pros: Zero new abstractions, 5-line change
- Cons: Detail views show slug (e.g. "improve-error-handling") not display name

**Option B — `useDocumentTitle` context (richer)**
Create a `DocumentTitleContext` with a `useDocumentTitle(contentName)` hook. Pages call it with their display name. AppShell reads the context value and combines with `projectName`.

- Pros: Pages can set rich names (e.g. feature title, ponder title)
- Cons: Every page needs a hook call, more plumbing

**? Open: Option A vs B?**

**Recommendation: Start with A, evolve to B if needed.** Option A is a 5-minute change that immediately solves the tab identification problem. The slug-based names are good enough for most cases. If users want display names later, Option B is an additive change.

### Mapping (Option A)

| Route | Title |
|-------|-------|
| `/` | `sdlc · Dashboard · Ponder` |
| `/features` | `sdlc · Features · Ponder` |
| `/features/auth-flow` | `sdlc · auth-flow · Ponder` |
| `/ponder` | `sdlc · Ponder · Ponder` |
| `/ponder/ui-title` | `sdlc · ui-title · Ponder` |
| `/milestones/v1` | `sdlc · v1 · Ponder` |

### Detail: `titleFromPath` Enhancement

The existing `titleFromPath` returns section labels for base routes. Extend it to return the slug for detail routes:

```typescript
function titleFromPath(pathname: string): string {
  if (PATH_LABELS[pathname]) return PATH_LABELS[pathname]
  for (const [path, label] of Object.entries(PATH_LABELS)) {
    if (path \!== "/" && pathname.startsWith(path + "/")) {
      const slug = pathname.slice(path.length + 1)
      return slug || label
    }
  }
  return "Dashboard"
}
```

Then in `AppShell`:
```typescript
useEffect(() => {
  const focus = titleFromPath(location.pathname)
  document.title = `${projectName} · ${focus} · Ponder`
}, [location.pathname, projectName])
```
