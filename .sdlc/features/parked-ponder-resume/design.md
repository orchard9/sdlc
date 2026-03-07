# Design: Parked Ponder Resume Button

## Overview

Add a Resume button to the ponder detail header that appears only when `entry.status === 'parked'`. It occupies the same slot as the Commit button (which is hidden when parked).

## Component Changes

### PonderPage.tsx — Detail Header (line ~500)

Current logic hides the Commit button when parked:
```tsx
{entry.status !== 'committed' && entry.status !== 'parked' && (
  <button onClick={handleCommit}>...</button>
)}
```

Add a Resume button in the complementary condition:
```tsx
{entry.status === 'parked' && (
  <button onClick={() => handleStatusChange('exploring')}>
    <Play className="w-3 h-3" />
    <span className="hidden sm:inline">Resume</span>
  </button>
)}
```

Styling: same shape as the Commit button — `shrink-0 flex items-center gap-1.5 px-2.5 py-1 text-xs font-medium rounded-lg border` — with a green accent (`bg-emerald-600 hover:bg-emerald-500 text-white border-emerald-600`) to differentiate from the violet Commit action.

### DialoguePanel.tsx — Empty State (line ~78)

Same pattern: when parked, show Resume instead of hiding all actions. Add a Resume button in the empty state that calls `handleStatusChange('exploring')` via a new `onResume` prop.

### No Backend Changes

The existing `PUT /api/roadmap/:slug` with `{ "status": "exploring" }` handles the transition. `handleStatusChange` in PonderPage already calls this endpoint.

## Data Flow

```
User clicks Resume
  → handleStatusChange('exploring')
    → PUT /api/roadmap/:slug { status: "exploring" }
      → ponder.update_status(Exploring)
      → state.yaml active_ponders updated
    → UI re-renders with status=exploring
    → Commit button, chat input, action buttons reappear
```

## Mockup

[Mockup](mockup.html)
