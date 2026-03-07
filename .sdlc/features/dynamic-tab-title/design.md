# Design: Dynamic Browser Tab Title

## Approach

Add a single `useEffect` in `AppShell` that updates `document.title` whenever `location.pathname` or `projectName` changes. Reuse the existing `titleFromPath()` helper already in `AppShell.tsx`.

## Title Construction

```
{projectName} · {focusLabel} · Ponder
```

Examples:
- Dashboard: `sdlc · Dashboard · Ponder`
- Milestones list: `sdlc · Milestones · Ponder`
- Feature detail `/features/dynamic-tab-title`: `sdlc · dynamic-tab-title · Features · Ponder`
- Milestone detail `/milestones/v50-ui-title`: `sdlc · v50-ui-title · Milestones · Ponder`
- Ponder workspace `/ponder/some-idea`: `sdlc · some-idea · Ponder · Ponder`

Before config loads (projectName not yet set): `Ponder · Dashboard · Ponder` (uses "Ponder" as fallback project name, which is already the default state).

## Implementation Detail

### `AppShell.tsx` change

Add after the existing `useEffect` blocks (~line 96):

```tsx
// Dynamic tab title: PROJECT · FOCUS · Ponder
useEffect(() => {
  const base = titleFromPath(location.pathname)
  const parts = location.pathname.split('/').filter(Boolean)
  // For detail routes like /features/:slug, prepend the slug
  const focus = parts.length >= 2 ? `${parts[parts.length - 1]} · ${base}` : base
  document.title = `${projectName} · ${focus} · Ponder`
}, [location.pathname, projectName])
```

### `HubPage.tsx` change

Add a `useEffect` at the top of the HubPage component:

```tsx
useEffect(() => {
  document.title = 'Ponder Hub'
}, [])
```

## Files Changed

1. `frontend/src/components/layout/AppShell.tsx` — add one `useEffect` (~5 lines)
2. `frontend/src/pages/HubPage.tsx` — add one `useEffect` (~3 lines)

## No New Dependencies

Zero new packages. Uses only `document.title` (Web API) and existing `titleFromPath()`.
