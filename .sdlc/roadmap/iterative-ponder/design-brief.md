# Design Brief

## The Ask
Add an **Iterate** button to released milestones and features that opens the NewIdeaModal pre-populated with:
- Title: original title (no version suffix in title)
- Slug: `{original-slug}-v2` / `-v3` / `-vN` (auto-incremented)
- Brief: references the original milestone/feature

## Key Discovery: NewIdeaModal Already Supports Pre-population
`NewIdeaModal` props already include `initialTitle?`, `initialSlug?`, `initialBrief?`. No modal changes needed — just pass the right values from the caller.

## Placement
1. **ReleasedPanel** (milestones) — alongside 'Re-run UAT' button
2. **FeatureDetail** released section — alongside release stats

## Versioning Logic
Need a utility: given a base slug, query existing ponders to find the next version number.
- No existing ponder: `{slug}-v2`
- `{slug}-v2` exists: `{slug}-v3`
- Pattern: strip trailing `-vN` to find the base, then scan all ponders for `{base}-vN` matches

## API Surface
No new endpoints needed. The existing `POST /api/roadmap` creates ponders. The existing `GET /api/roadmap?all=true` can be used client-side to detect version collisions.

⚑ Decided: Reuse NewIdeaModal with initial* props — no modal changes needed
⚑ Decided: Button text is 'Iterate'
? Open: Should the brief auto-reference the milestone vision, or just the slug/title?
? Open: Should we also show Iterate on the MilestonesPage archive cards, not just the detail panel?