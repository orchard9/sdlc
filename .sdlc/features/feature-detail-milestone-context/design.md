# Design: Feature Detail — Milestone Breadcrumb and Enhanced Done State

## Overview

Three changes to the feature detail page: (1) breadcrumb showing parent milestone, (2) richer done-state panel, (3) archived badge.

## Backend Change

### API: `GET /api/features/:slug` — add `milestone` field

In `crates/sdlc-server/src/routes/features.rs` → `get_feature()`:

After loading the feature, load `Milestone::list(&root)` and find the first milestone whose `features` vec contains this slug. Include in the JSON response:

```json
"milestone": { "slug": "v48-...", "title": "Milestone Title" }
// or
"milestone": null
```

This is a linear scan of milestones (typically <20). No index needed.

## Frontend Changes

### 1. TypeScript type update

In `frontend/src/lib/types.ts`, add to `FeatureDetail`:

```typescript
milestone: { slug: string; title: string } | null
```

### 2. Breadcrumb (replaces back link)

In `FeatureDetail.tsx`, replace the current `<Link to="/">Back</Link>` with a breadcrumb:

```tsx
<nav className="flex items-center gap-1.5 text-sm text-muted-foreground mb-4">
  {feature.milestone ? (
    <>
      <Link to="/milestones" className="hover:text-foreground transition-colors">Milestones</Link>
      <span>/</span>
      <Link to={`/milestones/${feature.milestone.slug}`} className="hover:text-foreground transition-colors">
        {feature.milestone.title}
      </Link>
      <span>/</span>
      <span className="text-foreground">{feature.title}</span>
    </>
  ) : (
    <>
      <Link to="/" className="hover:text-foreground transition-colors">Features</Link>
      <span>/</span>
      <span className="text-foreground">{feature.title}</span>
    </>
  )}
</nav>
```

### 3. Enhanced Done Panel

Replace the minimal green banner (lines 201-205) with:

```tsx
{classification?.action === 'done' && (
  <div className="bg-green-500/10 border border-green-500/30 rounded-xl p-4 mb-6">
    <div className="flex items-center gap-2 mb-2">
      <CheckCircle2 className="w-4 h-4 text-green-400" />
      <span className="text-sm font-medium text-green-400">Released</span>
    </div>
    <div className="flex items-center gap-4 text-xs text-muted-foreground">
      {releasedAt && <span>Released {formatRelative(releasedAt)}</span>}
      {journeyDays > 0 && <span>{journeyDays}d journey</span>}
      {feature.milestone && (
        <Link to={`/milestones/${feature.milestone.slug}`} className="hover:text-foreground transition-colors">
          {feature.milestone.title}
        </Link>
      )}
    </div>
  </div>
)}
```

Derive `releasedAt` from the last `phase_history` entry where `phase === 'released'`, and `journeyDays` from `created_at` to `releasedAt`.

### 4. Archived Badge

Next to the `StatusBadge` in the header, conditionally render:

```tsx
{feature.archived && (
  <span className="text-xs px-2 py-0.5 rounded bg-muted text-muted-foreground border border-border">Archived</span>
)}
```

## Files Modified

| File | Change |
|---|---|
| `crates/sdlc-server/src/routes/features.rs` | Add milestone lookup to `get_feature` |
| `frontend/src/lib/types.ts` | Add `milestone` field to `FeatureDetail` |
| `frontend/src/pages/FeatureDetail.tsx` | Breadcrumb, done panel, archived badge |

## No New Dependencies

Uses existing `Milestone::list()`, `lucide-react` icons (add `CheckCircle2` import), and standard date math.
