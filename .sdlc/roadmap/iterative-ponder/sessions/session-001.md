---
session: 1
timestamp: 2026-03-07T23:45:00Z
orientation:
  current: "Design is complete. All decisions resolved. Implementation plan is concrete with no unknowns."
  next: "Commit to milestones/features and implement: nextIterationSlug utility, Iterate button in ReleasedPanel + FeatureDetail."
  commit: "Met — the idea is fully shaped, scope is small, no new APIs needed, and the existing NewIdeaModal props already support pre-population."
---

**Xist · Owner**
iterative-ponder

when a feature or milestone has been released, add a ui action to allow the user to create a follow-up ponder session based on the item.

when creating the follow-up ponder session, how the new idea dialog, set the initial description to reference the original milestone or feature from which the button was clicked. set the title and slug to match the original, with an incremental "v2", "v3", "vN" at the end.

call the ability and button "Iterate".

---

## Discovery: NewIdeaModal Already Supports Pre-population

The existing `NewIdeaModal` component accepts `initialTitle?`, `initialSlug?`, and `initialBrief?` props. No modal changes are needed — the entire feature is about passing the right values from two new call sites.

## Roundtable

**Ben Hartley (Developer UX):** The moment a milestone ships is when you have the freshest context. Pre-populating removes the friction of manually opening a blank ponder and referencing the original. Key point: the title should NOT include the version suffix — "Git Status Indicator v2" reads like a product version, not an iteration. Keep the title clean; the slug carries the version for deduplication.

**Dana Cho (Product Skeptic):** The brief should be useful, not just a reference. Pull in the milestone vision so the user has a running start. Suggested brief template includes original vision and a prompt for "what worked, what to improve, what to explore next."

**Tobias Krenn (Engineering Skeptic):** Scope check — this is ~40 lines of new code across 3 files:
1. `nextIterationSlug()` utility — 10 lines
2. Iterate button in ReleasedPanel — ~15 lines
3. Iterate button in FeatureDetail — ~15 lines

No new API endpoints, no Rust changes, pure frontend. Satisfied.

**Dan Reeves (Systems Minimalist):** Considered skipping version detection and just appending `-v2`, letting the user fix collisions. Rejected — the whole point is reducing friction. The 10-line utility is worth it.

**Felix Wagner (Tooling Architect):** Edge cases for slug versioning:
- Base slug already ends in `-v2`: strip to find root, scan from there
- Multiple versions with gaps (v2, v4): use max+1, not next gap
- Slug length limit (40 chars): truncate base if needed

## Decisions

- **Title stays clean** — no version suffix in title, just the original title
- **Slug gets `-vN`** — auto-incremented by scanning existing ponder slugs
- **Brief includes vision** — template: original title, slug, vision text, prompt for reflection
- **Two placement points** — ReleasedPanel (milestones) + FeatureDetail released section (features)
- **No new API endpoints** — `api.getRoadmap()` provides existing slugs, `api.createPonderEntry()` creates the ponder
- **Button label: "Iterate"** — with RefreshCw icon, styled to match existing action buttons
- **Navigation** — after creation, `useNavigate` to `/ponder/{newSlug}`

## Implementation Plan

### New file: `frontend/src/lib/iterate.ts`
```typescript
export function nextIterationSlug(slug: string, existingSlugs: string[]): string {
  const base = slug.replace(/-v\d+$/, '')
  const pattern = new RegExp(`^${base}-v(\\d+)$`)
  let maxVersion = 1
  for (const s of existingSlugs) {
    const match = s.match(pattern)
    if (match) maxVersion = Math.max(maxVersion, parseInt(match[1], 10))
  }
  return `${base}-v${maxVersion + 1}`.slice(0, 40)
}
```

### Modified: `frontend/src/components/milestones/ReleasedPanel.tsx`
- Add Iterate button in actions row (next to Re-run UAT)
- Fetch ponder slugs via `api.getRoadmap(true)`
- Open NewIdeaModal with milestone title, next version slug, vision-based brief
- On created, navigate to `/ponder/{slug}`

### Modified: `frontend/src/pages/FeatureDetail.tsx`
- Add Iterate button in released panel (lines 221-243)
- Same pattern: fetch ponder slugs, open modal, navigate

### Brief Templates

Milestone:
```
Iteration of milestone: {title} ({slug})

Original vision:
{vision || 'No vision recorded.'}

What worked well, what to improve, and what to explore next:
```

Feature:
```
Iteration of feature: {title} ({slug})

{description || ''}

What worked well, what to improve, and what to explore next:
```

## Commit Signal: MET

The idea is fully shaped. Scope is small (~40 lines across 3 files), no new APIs, the existing modal props already support it, and all edge cases are accounted for. Ready to commit to milestones/features.
