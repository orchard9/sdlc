# Design: Ponder-First Entry Path for New Users

## Overview

This feature makes three surgical changes to the frontend:

1. **Remove the automatic redirect to `/setup`** — the Dashboard's `setupIncomplete` banner stays as an informational notice, but we never force-navigate the user away.
2. **Add explanatory subtitles** to VisionPage and ArchitecturePage headings.
3. **Auto-open the NewIdeaForm** on PonderPage when the URL contains `?new=1`.

No backend changes are required. All work is confined to three React components.

## File Map

| File | Change |
|---|---|
| `frontend/src/pages/Dashboard.tsx` | Remove any hard redirect / `useNavigate` push to `/setup` on first load |
| `frontend/src/pages/VisionPage.tsx` | Add subtitle under heading |
| `frontend/src/pages/ArchitecturePage.tsx` | Add subtitle under heading |
| `frontend/src/pages/PonderPage.tsx` | Detect `?new=1` and call `setShowForm(true)` on mount |

## Investigation Findings

Reviewing the existing code:

- **Dashboard.tsx** — uses `setSetupIncomplete(true)` to conditionally render a banner with a "Go to Setup →" link. There is no `useNavigate` push. No hard redirect exists. The banner is already purely informational. **No redirect removal needed** — the existing behavior already satisfies the spec. The banner content may optionally be softened to add a "New Ponder" CTA alongside "Go to Setup".
- **VisionPage.tsx** — heading is `<h2 className="text-xl font-semibold">Vision</h2>` with no subtitle. Simple one-line addition.
- **ArchitecturePage.tsx** — heading is `<h2 className="text-xl font-semibold">Architecture</h2>` with no subtitle. Simple one-line addition.
- **PonderPage.tsx** — `showForm` state controls the `NewIdeaForm`. There is no `?new=1` detection. `useSearchParams` from react-router-dom is available; detect it on mount.
- **server-side** — no `setup_complete` enforcement anywhere in the codebase. No change needed.

## Wireframes

### Dashboard Banner (unchanged layout, CTA text adjusted)

```
┌─────────────────────────────────────────────────────────────────┐
│  ⚠  Project setup is incomplete — agents won't have enough      │
│     context to work with.          [New Ponder]  [Go to Setup→] │
└─────────────────────────────────────────────────────────────────┘
```

The existing banner already doesn't force-redirect. We add a secondary "New Ponder" CTA that links to `/ponder?new=1`.

### Vision Page (subtitle added)

```
  🎯  Vision                                       [Align ✦]
      What you're building and why — agents use
      this to make the right tradeoffs.
  ──────────────────────────────────────────────────────────
  <document content / empty state>
```

### Architecture Page (subtitle added)

```
  ⎇  Architecture                                  [Align ✦]
     How it's built — agents use this to write
     code that fits the system.
  ──────────────────────────────────────────────────────────
  <document content / empty state>
```

### Ponder Page with `?new=1`

```
  ┌─────────────────────────────────────────────────────────┐
  │ Ponder             [✦] [+]                              │
  ├─────────────────────────────────────────────────────────┤
  │ ┌─── New Idea ─────────────────────────────── [×] ───┐  │
  │ │ What are you thinking about?                       │  │
  │ │ slug                                               │  │
  │ │ Brief description (optional)                       │  │
  │ │                         [Cancel]  [Create]         │  │
  │ └────────────────────────────────────────────────────┘  │
  │  All  Exploring  Converging  Committed  Parked          │
  │  ─────────────────────────────────────────────────────  │
  │  (empty list)                                           │
  └─────────────────────────────────────────────────────────┘
```

The NewIdeaForm appears automatically — the user lands straight in the "idea capture" state.

## Implementation Details

### 1. Dashboard.tsx — Add "New Ponder" CTA to setup banner

In the `setupIncomplete` banner, add a `Link to="/ponder?new=1"` as a secondary action alongside the existing "Go to Setup →" link.

```tsx
{setupIncomplete && (
  <div className="flex items-center justify-between gap-3 px-4 py-3 mb-6 rounded-lg border border-amber-500/30 bg-amber-950/20 text-sm">
    <span className="text-amber-300/80">
      Project setup is incomplete — agents won't have enough context to work with.
    </span>
    <div className="flex items-center gap-3 shrink-0">
      <Link
        to="/ponder?new=1"
        className="text-amber-300/70 hover:text-amber-200 transition-colors whitespace-nowrap"
      >
        New Ponder
      </Link>
      <Link
        to="/setup"
        className="text-amber-300 hover:text-amber-200 transition-colors whitespace-nowrap font-medium"
      >
        Go to Setup →
      </Link>
    </div>
  </div>
)}
```

### 2. VisionPage.tsx — Subtitle

Below the `<h2>` heading, add:

```tsx
<p className="text-sm text-muted-foreground mt-0.5">
  What you're building and why — agents use this to make the right tradeoffs.
</p>
```

### 3. ArchitecturePage.tsx — Subtitle

Below the `<h2>` heading, add:

```tsx
<p className="text-sm text-muted-foreground mt-0.5">
  How it's built — agents use this to write code that fits the system.
</p>
```

### 4. PonderPage.tsx — `?new=1` auto-open

Add `useSearchParams` import. In the `PonderPage` component body, after the existing state declarations, add:

```tsx
const [searchParams, setSearchParams] = useSearchParams()

useEffect(() => {
  if (searchParams.get('new') === '1' && !showForm) {
    setShowForm(true)
    setSearchParams({}, { replace: true })
  }
}, []) // run once on mount
```

The `setSearchParams({}, { replace: true })` clears the `?new=1` param from the URL after triggering the form, so the browser history is clean and a manual page refresh doesn't re-open the form.

## Constraints

- No new components — all changes are inline edits to existing pages.
- No backend changes.
- The Setup page remains fully accessible from the sidebar.
- The "Go to Setup" link in the banner is preserved; it gains a sibling "New Ponder" link.
