# Tasks: Ponder-First Entry Path for New Users

## T1 — Dashboard: Add "New Ponder" CTA to setup-incomplete banner

**File:** `frontend/src/pages/Dashboard.tsx`

In the `setupIncomplete` banner, wrap the existing "Go to Setup →" link in a flex container and add a secondary "New Ponder" link that navigates to `/ponder?new=1`.

**Acceptance:**
- Banner contains both "New Ponder" and "Go to Setup →" links when `setupIncomplete` is true.
- "New Ponder" link `href` is `/ponder?new=1`.
- Banner still does not force-redirect the user automatically.

---

## T2 — VisionPage: Add explanatory subtitle

**File:** `frontend/src/pages/VisionPage.tsx`

Add a `<p>` subtitle below the `<h2>Vision</h2>` heading:

> "What you're building and why — agents use this to make the right tradeoffs."

Styling: `text-sm text-muted-foreground mt-0.5`

**Acceptance:**
- Subtitle is visible on the Vision page below the heading.
- Subtitle text matches spec exactly.

---

## T3 — ArchitecturePage: Add explanatory subtitle

**File:** `frontend/src/pages/ArchitecturePage.tsx`

Add a `<p>` subtitle below the `<h2>Architecture</h2>` heading:

> "How it's built — agents use this to write code that fits the system."

Styling: `text-sm text-muted-foreground mt-0.5`

**Acceptance:**
- Subtitle is visible on the Architecture page below the heading.
- Subtitle text matches spec exactly.

---

## T4 — PonderPage: Auto-open NewIdeaForm on `?new=1`

**File:** `frontend/src/pages/PonderPage.tsx`

1. Add `useSearchParams` to the react-router-dom import.
2. In `PonderPage`, declare `const [searchParams, setSearchParams] = useSearchParams()`.
3. Add a `useEffect` that fires once on mount: if `searchParams.get('new') === '1'`, call `setShowForm(true)` and clear the query param with `setSearchParams({}, { replace: true })`.

**Acceptance:**
- Navigating to `/ponder?new=1` automatically shows the NewIdeaForm without the user clicking "+".
- After form auto-opens, the URL becomes `/ponder` (param cleared).
- Refreshing `/ponder` (no param) does not re-open the form.
- Manual "+" button still works normally.
