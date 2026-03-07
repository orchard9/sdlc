# Tasks: Parked Ponder Resume Button

## Task 1: Add Resume button to PonderPage detail header

**File:** `frontend/src/pages/PonderPage.tsx`

After the existing Commit button block (~line 500), add a complementary block:

```tsx
{entry.status === 'parked' && (
  <button
    onClick={() => handleStatusChange('exploring')}
    className="shrink-0 flex items-center gap-1.5 px-2.5 py-1 text-xs font-medium rounded-lg border bg-emerald-600 hover:bg-emerald-500 text-white border-emerald-600 transition-colors"
    title="Resume exploring this ponder"
  >
    <Play className="w-3 h-3" />
    <span className="hidden sm:inline">Resume</span>
  </button>
)}
```

`Play` is already imported. `handleStatusChange` already exists and calls the PUT endpoint.

## Task 2: Add Resume button to DialoguePanel empty state

**File:** `frontend/src/components/ponder/DialoguePanel.tsx`

Add an `onResume` callback prop. In the empty state, when `entry.status === 'parked'`, show a Resume button instead of hiding all actions:

```tsx
{entry.status === 'parked' && onResume && (
  <button onClick={onResume} className="...btn-resume...">
    <Play className="w-3 h-3" />
    Resume exploring
  </button>
)}
```

Pass `onResume={() => handleStatusChange('exploring')}` from PonderPage when rendering DialoguePanel.

## Task 3: Verify Play icon import in DialoguePanel

Ensure `Play` from `lucide-react` is imported in `DialoguePanel.tsx`. Add to imports if missing.
