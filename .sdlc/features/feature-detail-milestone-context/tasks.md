# Tasks: Feature Detail — Milestone Breadcrumb and Enhanced Done State

## T1: Add milestone lookup to GET /api/features/:slug

In `crates/sdlc-server/src/routes/features.rs`, modify `get_feature()` to load all milestones via `Milestone::list()`, find the one containing this feature slug, and include `"milestone": { "slug", "title" } | null` in the JSON response.

## T2: Update FeatureDetail TypeScript type

Add `milestone: { slug: string; title: string } | null` to the `FeatureDetail` interface in `frontend/src/lib/types.ts`.

## T3: Replace back link with milestone breadcrumb

In `FeatureDetail.tsx`, replace the `<Link to="/">Back</Link>` with a breadcrumb nav showing `Milestones > [Milestone Title] > [Feature Title]` when a milestone exists, or `Features > [Feature Title]` otherwise. All segments except the last are clickable links.

## T4: Enhanced done-state panel

Replace the minimal green banner with a richer panel showing: green CheckCircle2 icon, "Released" label, release date (from phase_history), journey duration (created_at to released), and parent milestone link.

## T5: Archived badge

Add a muted "Archived" badge next to the phase StatusBadge when `feature.archived === true`.
