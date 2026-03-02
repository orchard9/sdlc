## Summary

Two-part change: (1) extend `ArtifactViewer.tsx` to show a 120-char teaser and relative timestamp in the artifact card header, using a pure client-side extraction function on `artifact.content`; (2) add a `## Summary` convention to the `sdlc-run` and `sdlc-next` command templates in `crates/sdlc-cli/src/cmd/init/commands/sdlc_run.rs` and `sdlc_next.rs` so agents write summaries that become meaningful teaser content.

## Scope

- `frontend/src/components/features/ArtifactViewer.tsx` — card header extended with timestamp + teaser row
- `crates/sdlc-cli/src/cmd/init/commands/sdlc_run.rs` — `## Summary` instruction injected into `SDLC_RUN_COMMAND` and `SDLC_RUN_PLAYBOOK`
- `crates/sdlc-cli/src/cmd/init/commands/sdlc_next.rs` — same injection into `SDLC_NEXT_COMMAND` and `SDLC_NEXT_PLAYBOOK`

No Rust type changes, no new API fields, no backend changes.

## Part A: Artifact Card Teaser (Frontend)

### Timestamp Logic

Use `artifact.approved_at` as the primary timestamp. Format with a lightweight custom utility — no new dependency. Implement a standalone `formatRelativeTime(iso: string): string` function.

### Teaser Extraction Logic

```ts
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

Priority: if `## Summary` section is found, its first non-empty paragraph is used. Otherwise, the first non-heading body paragraph is used.

## Part B: ## Summary Convention in Agent Instructions

### Files to Edit

- `crates/sdlc-cli/src/cmd/init/commands/sdlc_run.rs`
- `crates/sdlc-cli/src/cmd/init/commands/sdlc_next.rs`

In `SDLC_RUN_COMMAND` and `SDLC_NEXT_COMMAND`, add the `## Summary` instruction to the artifact creation section. In playbook variants, add a bullet directing agents to begin every artifact with `## Summary`.

## What is NOT Changing

- No new API fields on `Artifact`
- No changes to `sdlc-core` types
- No changes to `sdlc-server` routes
- No changes to artifact storage format
- No LLM calls for summary generation
