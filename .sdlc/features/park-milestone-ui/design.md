# Design — Park/Unpark UI

## Overview

Minimal UI changes across four components to support the `parked` milestone status.

## Component Changes

### 1. `types.ts` — MilestoneStatus

Add `'parked'` to the union type:
```typescript
export type MilestoneStatus = 'active' | 'verifying' | 'released' | 'skipped' | 'parked'
```

### 2. `client.ts` — API Methods

```typescript
parkMilestone: (slug: string) =>
  request(`/api/milestones/${encodeURIComponent(slug)}/park`, { method: 'PATCH' }),
unparkMilestone: (slug: string) =>
  request(`/api/milestones/${encodeURIComponent(slug)}/unpark`, { method: 'PATCH' }),
```

### 3. `MilestonesPage.tsx` — Three Sections

Current: `active` (non-released) and `released` (archive).

New filtering logic:
- **Active**: `status !== 'released' && status !== 'skipped' && status !== 'parked'`
- **Parked**: `status === 'parked'` — collapsed by default, same UX pattern as Archive
- **Archive**: `status === 'released' || status === 'skipped'`

Parked section uses the same `ChevronDown`/`ChevronRight` toggle pattern as Archive. Shows count: "Parked (N)".

### 4. `HorizonZone.tsx` — Filter Parked

Add to the existing `horizonMilestones` filter:
```typescript
const horizonMilestones = milestones.filter(m => {
  if (m.status === 'parked') return false  // <-- new
  if (m.features.length === 0) return true
  return m.features.every(slug => {
    const f = featureBySlug.get(slug)
    return !f || f.phase === 'draft'
  })
})
```

### 5. `CurrentZone.tsx` — Exclude Parked

The `CurrentZone` receives milestones already filtered by the Dashboard parent. However, as a defensive measure, milestones passed to CurrentZone should not include parked ones. The parent `Dashboard.tsx` already does the filtering — we just need to ensure parked milestones are excluded in the same place where released ones are.

### 6. `MilestoneDetail.tsx` — Park/Unpark Button

Add a button in the header area next to the status badge:
- If `status === 'active' || status === 'verifying'`: show "Park" button (muted style)
- If `status === 'parked'`: show "Unpark" button (primary style)
- If `status === 'released' || status === 'skipped'`: no button (terminal states)

On click: call `api.parkMilestone(slug)` or `api.unparkMilestone(slug)`, then reload.

## No Mockup Needed

The changes are minimal filter adjustments and one button addition. The visual patterns (collapsible sections, status badges) already exist in the codebase. A mockup would add no value.
