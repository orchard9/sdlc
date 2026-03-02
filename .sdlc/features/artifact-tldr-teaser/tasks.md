## Summary

Four implementation tasks: add `extractTeaser` and `formatRelativeTime` utilities to `ArtifactViewer.tsx`, render the teaser row in the card header, then inject the `## Summary` artifact convention into the `sdlc-run` and `sdlc-next` command templates. All changes are additive and non-breaking.

## Tasks

### T1: Add utility functions to ArtifactViewer.tsx

Add `extractTeaser(content: string): string` and `formatRelativeTime(iso: string): string` as module-level functions in `frontend/src/components/features/ArtifactViewer.tsx`.

Acceptance:
- `extractTeaser` skips the H1 line, prefers `## Summary` section content, falls back to first body paragraph, truncates at 120 chars with `…`
- `formatRelativeTime` converts ISO string to "Xs ago", "Xm ago", "Xh ago", "Xd ago", or "over a month ago"
- Both functions are pure (no side effects, no imports needed)

### T2: Render teaser row in ArtifactViewer card header

In `ArtifactViewer.tsx`, add a second row below the existing header row that shows the timestamp (from `artifact.approved_at`) and extracted teaser text.

Acceptance:
- Import `Clock` from `lucide-react`
- The teaser row is only rendered when `artifact.content` is non-null and at least one of teaser/timestamp is non-empty
- Timestamp uses `Clock` icon + relative time text
- Separator `·` appears only when both timestamp and teaser are present
- Teaser text is italicized and quoted
- Row uses `text-xs text-muted-foreground` styling consistent with the rest of the card

### T3: Inject ## Summary convention into sdlc-next templates

In `crates/sdlc-cli/src/cmd/init/commands/sdlc_next.rs`, add the `## Summary` instruction text to `SDLC_NEXT_COMMAND` (Claude format) and `SDLC_NEXT_PLAYBOOK` (Gemini/OpenCode format).

Acceptance:
- `SDLC_NEXT_COMMAND`: after "3. Write a thorough Markdown artifact to `output_path`", add a blockquote with `## Summary` instruction
- `SDLC_NEXT_PLAYBOOK`: after "- Write the required artifact to `output_path`.", add a bullet about the convention
- `SDLC_NEXT_SKILL` (Agents format) is not modified

### T4: Inject ## Summary convention into sdlc-run templates

In `crates/sdlc-cli/src/cmd/init/commands/sdlc_run.rs`, add a brief convention callout to `SDLC_RUN_COMMAND` and a step to `SDLC_RUN_PLAYBOOK`.

Acceptance:
- `SDLC_RUN_COMMAND`: in "### 3. Run the loop", add a brief note about the `## Summary` convention
- `SDLC_RUN_PLAYBOOK`: add step 5 for the convention, renumber subsequent steps accordingly
- `SDLC_RUN_SKILL` (Agents format) is not modified
