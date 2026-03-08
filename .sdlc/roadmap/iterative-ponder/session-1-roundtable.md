# Session 1: Roundtable

**Xist · Owner**
iterative-ponder

when a feature or milestone has been released, add a ui action to allow the user to create a follow-up ponder session based on the item.

when creating the follow-up ponder session, how the new idea dialog, set the initial description to reference the original milestone or feature from which the button was clicked. set the title and slug to match the original, with an incremental "v2", "v3", "vN" at the end.

call the ability and button "Iterate".

---

**Ben Hartley · Developer UX**
This is a great affordance. The moment a milestone ships is exactly when you have the freshest context about what worked and what didn't. The cognitive cost of opening a blank ponder and manually referencing the original is just high enough that people won't do it. Pre-populating removes that friction.

One concern: the title should NOT include the version suffix. "Git Status Indicator v2" reads like a product version, not an iteration. Keep the title clean — the slug carries the version for deduplication. The brief is where lineage lives.

**Dana Cho · Product Skeptic**
I like it but let me push: what goes in the brief? Just "Follow-up to milestone: git-status-indicator"? That's thin. The brief should be *useful* — pull in the milestone vision and any open tasks that didn't get addressed. Give the user a running start.

Suggested brief template:
```
Iteration of milestone: {title} ({slug})

Original vision: {vision}

Open questions from the previous cycle:
- [user fills in or we auto-populate from remaining tasks]
```

**Tobias Krenn · Engineering Skeptic**
How much are we building here? Let me count:
1. A `nextVersionSlug(baseSlug, existingPonders)` utility — 10 lines
2. An Iterate button in ReleasedPanel — ~15 lines
3. An Iterate button in FeatureDetail released section — ~15 lines
4. State management: `useState` for modal open, pass initial* props — trivial

This is a small, clean feature. No new API endpoints, no Rust changes, pure frontend. I'm satisfied with the scope.

**Dan Reeves · Systems Minimalist**
One primitive question: do we need version detection at all? Could we just always append `-v2` and let the API reject duplicates? Then the user manually bumps to `-v3` in the slug field if needed. The modal already lets you edit the slug.

...Actually, no. The whole point is reducing friction. If the user has to fix a collision, that's worse than auto-detecting. The utility is worth the 10 lines.

**Felix Wagner · Tooling Architect**
The slug versioning utility should handle edge cases:
- Base slug already ends in `-v2`: strip to find root, scan from there
- Multiple versions exist with gaps (v2, v4): use max+1, not next gap
- Slug length limit (40 chars): if adding `-vN` exceeds 40, truncate the base

⚑ Decided: Title stays clean (no version suffix), slug gets `-vN`
⚑ Decided: Brief includes original title, slug, and vision text
⚑ Decided: Auto-detect next version number from existing ponder slugs
⚑ Decided: Two placement points: ReleasedPanel + FeatureDetail released section
⚑ Decided: No new API endpoints — pure frontend feature
? Open: Should we show Iterate on milestone archive cards in MilestonesPage too?
? Resolved: Yes — but defer to v1 of the feature. Start with the detail views.