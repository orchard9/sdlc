# Tasks: Dynamic Browser Tab Title

## T1: Add useEffect for document.title in AppShell

**File**: `frontend/src/components/layout/AppShell.tsx`

Add a `useEffect` that constructs and sets `document.title` using the pattern `{projectName} · {focus} · Ponder`. For detail routes (2+ path segments), prepend the slug to the base label. Depends on `location.pathname` and `projectName`.

## T2: Set title in HubPage

**File**: `frontend/src/pages/HubPage.tsx`

Add `useEffect(() => { document.title = 'Ponder Hub' }, [])` to the HubPage component.

## T3: Verify titles across all routes

Manual verification that all major routes produce correct titles:
- `/` -> `{project} · Dashboard · Ponder`
- `/milestones` -> `{project} · Milestones · Ponder`
- `/features/some-slug` -> `{project} · some-slug · Features · Ponder`
- `/ponder/some-idea` -> `{project} · some-idea · Ponder · Ponder`
- Hub mode -> `Ponder Hub`
