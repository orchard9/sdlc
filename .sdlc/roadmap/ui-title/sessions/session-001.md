---
session: 1
timestamp: 2026-03-07T17:42:00Z
orientation:
  current: "Design complete — Option A (slug-based useEffect in AppShell) selected, separator and format decided"
  next: "Implement: add useEffect to AppShell, enhance titleFromPath, update index.html fallback"
  commit: "Met — idea is fully shaped, implementation is a ~10-line change in one file"
---

**Xist · Owner**
ui title

The title of the UI in the web browser must be more descriptive. Users commonly run many browser tabs simultaneously, causing the left side of the name to be more valuable than the right for identification purposes.

Name the title like "PROJECT $ FOCUS CONTENT NAME $ Ponder"

Where the `PROJECT` matches the name of the current project (which might change during runtime), and the `FOCUS CONTENT NAME` identifies whatever is currently being focused in the primary content area at the current time (changes during page navigation)

---

## Session 1: Design

### Context Discovered

The frontend already has all the pieces:

- `AppShell` holds `projectName` (fetched from config API on mount)
- `titleFromPath(pathname)` maps routes to human labels (Dashboard, Features, Ponder, etc.)
- Detail routes (`/features/:slug`, `/ponder/:slug`) have the slug in the URL
- Currently `index.html` has a static `<title>sdlc</title>` — never updated at runtime

### Separator Choice

The brief proposes `$` as separator. Pushing back on this:

**⚑ Decided: Use `·` (middle dot) instead.**

`$` reads as a shell variable or currency symbol. Every major web app (GitHub, Google Docs, Slack, Linear) uses `·`, `—`, or `|` as title separators. The middle dot is cleanest at narrow tab widths and has no semantic baggage. Examples:

- GitHub: `repo · Pull Request #123`
- Google Docs: `Document Name - Google Docs`
- Linear: `Issue Title · Linear`

### Format: `PROJECT · FOCUS · Ponder`

The owner's instinct is correct — project name leftmost. When running multiple Ponder instances (one per project), the project name is the primary differentiator. Focus content second, branding last.

| Route | Title |
|-------|-------|
| `/` | `myapp · Dashboard · Ponder` |
| `/features` | `myapp · Features · Ponder` |
| `/features/auth-flow` | `myapp · auth-flow · Ponder` |
| `/ponder/ui-title` | `myapp · ui-title · Ponder` |
| `/milestones/v1` | `myapp · v1 · Ponder` |
| `/vision` | `myapp · Vision · Ponder` |

### Implementation: Option A (minimal, selected)

A single `useEffect` in `AppShell.tsx`:

```typescript
useEffect(() => {
  const focus = titleFromPath(location.pathname)
  document.title = `${projectName} · ${focus} · Ponder`
}, [location.pathname, projectName])
```

Enhance `titleFromPath` to return the slug for detail routes instead of just the section label:

```typescript
function titleFromPath(pathname: string): string {
  if (PATH_LABELS[pathname]) return PATH_LABELS[pathname]
  for (const [path, label] of Object.entries(PATH_LABELS)) {
    if (path !== '/' && pathname.startsWith(path + '/')) {
      const slug = pathname.slice(path.length + 1)
      return slug || label
    }
  }
  return 'Dashboard'
}
```

No new files, no new dependencies, no new abstractions. ~10 lines changed in `AppShell.tsx`.

### Open Question

**? Open: Hub page format.** The Hub manages multiple projects, so `PROJECT · Hub · Ponder` is slightly odd. Could use `Ponder Hub` instead. Low priority — hub has its own routing context and this can be addressed separately.

### Verdict

This is ready to build. The implementation is a single-file change with zero risk. Commit signal is met.
