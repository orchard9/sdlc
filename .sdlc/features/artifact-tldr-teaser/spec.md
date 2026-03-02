## Summary

Two coordinated changes: (1) add a 120-char teaser line and last-modified timestamp to each artifact card in `ArtifactViewer.tsx` so users get instant orientation before scrolling, and (2) add a `## Summary` convention to `sdlc-run` and `sdlc-next` agent instructions in `init.rs` so that teaser content is consistently meaningful. No new API fields needed — teaser is extracted client-side from existing `artifact.content`.

## Problem

The artifact card in `FeatureDetail.tsx` shows: artifact type label + status badge + fullscreen button. No preview of content. Users must scroll into a (previously constrained, now unconstrained per Feature 1) document to understand what it contains. For a spec that was last updated 2 hours ago by an agent run, users have no orientation without reading it.

## Solution

### Part A: Card Teaser in `ArtifactViewer.tsx`

Extend the artifact card header to show:
1. **Last-modified timestamp** — human-readable relative time (e.g., "12m ago", "2h ago"). Use `artifact.updated_at` or the last modification time from the artifact metadata. Format with a lightweight relative-time utility (e.g., `date-fns/formatDistanceToNow` or a simple custom function — no new dependency if date-fns already present, otherwise implement ~10 lines).

2. **120-char teaser** — extracted client-side from `artifact.content`:
   - Strip leading `# Heading` (H1)
   - If next non-empty block is `## Summary`, use that section's first paragraph
   - Otherwise use the first non-empty paragraph
   - Truncate to 120 characters with `…` suffix

```tsx
// Teaser extraction utility (client-side, in ArtifactViewer.tsx or a utils file)
function extractTeaser(content: string): string {
  const lines = content.split('\n');
  let inSummary = false;
  let skippedH1 = false;
  for (const line of lines) {
    const trimmed = line.trim();
    if (!skippedH1 && trimmed.startsWith('# ')) { skippedH1 = true; continue; }
    if (trimmed.startsWith('## Summary')) { inSummary = true; continue; }
    if (inSummary && trimmed && !trimmed.startsWith('#')) {
      return trimmed.length > 120 ? trimmed.slice(0, 117) + '…' : trimmed;
    }
    if (!inSummary && skippedH1 && trimmed && !trimmed.startsWith('#')) {
      return trimmed.length > 120 ? trimmed.slice(0, 117) + '…' : trimmed;
    }
  }
  return '';
}
```

The artifact card header becomes:
```
[spec] [Approved ✓]  [⤢ fullscreen]
12m ago · "The plan adopts a two-phase agent workflow with Plan→Act pattern..."
```

### Part B: `## Summary` Convention in Agent Instructions

In `crates/sdlc-cli/src/cmd/init.rs`, add to `SDLC_RUN_COMMAND` and `SDLC_NEXT_COMMAND` template text (in the artifact writing instructions section):

> Every spec, design, and tasks artifact you write must begin with a `## Summary` section (2-4 sentences) stating the current state of the plan — what has been decided, what the approach is, and what changed in this revision. This summary will be surfaced in the UI as a preview teaser card. Place `## Summary` immediately after the `# Title` line, before any other sections.

This is a 3-4 line addition to the existing agent instruction text. It requires no Rust type changes — it's purely instruction content.

## Acceptance Criteria

- An artifact card in `FeatureDetail.tsx` shows last-modified timestamp ("12m ago" format)
- An artifact card shows a 120-char teaser below the header row
- For an artifact beginning with `## Summary`, the teaser shows the summary content
- For an artifact without `## Summary`, the teaser shows the first body paragraph
- For a very short artifact, the teaser shows full content (no truncation if under 120 chars)
- `sdlc update` installs updated `sdlc-run` and `sdlc-next` commands with the `## Summary` instruction
- Agents running `sdlc-run` after the update write `## Summary` as the first section of new artifacts

## Files Changed

- `frontend/src/components/features/ArtifactViewer.tsx` — teaser and timestamp in card header
- `crates/sdlc-cli/src/cmd/init.rs` — `## Summary` convention added to `SDLC_RUN_COMMAND` and `SDLC_NEXT_COMMAND`

## What is NOT in scope

- Generating a summary via LLM for artifacts that don't have one
- Editing the teaser inline
- A separate "summary artifact" type
- Changing the artifact storage model
