# Artifact Viewer — Feature Direction

**Status:** Direction reached. Ready for feature creation and spec.
**Session:** 2 (2026-03-01)

---

## Recommended V1 Approach

The artifact viewer is not a new feature to build from scratch. It is a targeted improvement to `ArtifactViewer.tsx` (for feature artifacts) and `MarkdownContent.tsx` (the shared rendering core). The ponder scrapbook (`WorkspacePanel.tsx`) and the feature artifact viewer remain separate components that share a rendering layer.

**V1 delivers five changes in two weeks:**

### Change 1: Remove the height cap (Week 1, Day 1-2)

**File:** `frontend/src/components/features/ArtifactViewer.tsx`

Remove `max-h-96 overflow-y-auto` from the artifact content div. Let the artifact card expand to its natural height. The page itself scrolls — there is no need for a double scroll bar inside a fixed-height container inside a scrolling page.

This single change eliminates the primary source of Xist's complaint: "I can't really see that." A 400-line plan artifact will now be fully visible by scrolling the page normally.

### Change 2: File path auto-detection (Week 1, Day 3-4)

**File:** `frontend/src/components/shared/MarkdownContent.tsx`

In the inline `code` component handler (which already handles `` `backtick spans` ``), add a check: if the content matches a file path pattern (`/^[a-z_.][a-z0-9_\-./]*\.[a-z]{2,5}$/i`), render it as an `<a href="{ide}://file/{project_root}/{path}">` link instead of a code span.

**Required additions:**
- Add `project_root: string` to the API response (either `/api/state` or a new `/api/project` endpoint). The Rust side: add `project_root` field (the current working directory at server startup) to the state response struct.
- Add `ide_uri_scheme` to `.sdlc/config.yaml` settings, default `vscode`. Server reads and exposes it via API.
- Frontend reads both values and passes them into `MarkdownContent` as props.

This eliminates the second half of Xist's workaround: "I always make Agy include links in my plans, so all source files/classes named in the plans are linked to their actual source location."

### Change 3: Card teaser in ArtifactViewer (Week 1, Day 4-5)

**File:** `frontend/src/components/features/ArtifactViewer.tsx`

The artifact card header currently shows: artifact type + status badge + fullscreen button. Add: last-modified timestamp (human-readable, relative) + a single-line teaser (first 120 chars of body text below the leading `# Heading`).

The teaser extraction is client-side: strip the first `# Heading` line, take the next non-empty line or paragraph, truncate to 120 chars. If the content begins with `## Summary`, show the Summary content. This degrades gracefully for artifacts that don't follow the convention.

### Change 4: `## Summary` convention in agent instruction (Week 2, Day 1)

**File:** `crates/sdlc-cli/src/cmd/init.rs`

Add to `SDLC_RUN_COMMAND` (and `SDLC_NEXT_COMMAND`) agent instructions:

> Every spec, design, and tasks artifact you write must begin with a `## Summary` section (2-4 sentences) stating the current state of the plan — what has been decided, what the approach is, and what changed in this revision. This summary will be surfaced in the UI as a preview card.

This is a 3-line change that makes the card teaser feature meaningful for all future agent runs.

### Change 5: TOC in fullscreen view (Week 2, Day 2-4)

**Files:** `frontend/src/components/shared/MarkdownContent.tsx`, `frontend/src/components/shared/FullscreenModal.tsx`

When `MarkdownContent` is rendered in fullscreen mode:
- Extract all `#`, `##`, `###` headings during render, assign stable slug-based IDs
- Render a sticky left-rail TOC panel (`w-48`, `overflow-y-auto`, `sticky top-0`)
- TOC entries click-scroll to the heading anchor
- On narrow screens (below `lg:` breakpoint): TOC collapses to a "Jump to..." `<select>` dropdown at the top of the content

`FullscreenModal.tsx` gets a `hasToc?: boolean` prop and switches from a single-column layout to a two-column layout (`flex flex-row`) when a TOC is present.

---

## What the MVP Looks Like (End of Week 2)

A user opens a feature with a 400-line plan artifact. They see:
- The artifact card expanded to full height, no scrolling required within the card
- A teaser line: "12m ago · The plan adopts a two-phase agent workflow..."
- File references like `` `crates/sdlc-core/src/feature.rs` `` are clickable links that open VS Code
- Clicking the fullscreen button opens a full-viewport view with a sticky left-rail TOC
- Navigation between headings is instant; they do not need to scroll to find "Implementation Notes"

Xist's workaround is gone. There is no reason to export to `.md` and open in Agy.

---

## What is Deferred to V2

### Annotation/commenting model

Not built in V1. Designed here for future implementation:

The correct V1 of commenting (when validated) is:
1. User selects text in the fullscreen artifact view → "+comment" button appears
2. Click opens a sidebar annotation entry: quoted text + freeform note
3. User adds N annotations (accumulating state in React `useState`)
4. "Submit feedback" converts all annotations to a single agent prompt pre-filled with: "Feedback on [spec]: [for each annotation: quoted text + note]"
5. Annotations are ephemeral — not persisted, not stored in Rust — they live in React state until submitted

Data model for this: no new Rust types needed. The feedback submission creates a task or opens a chat input pre-filled with the text. This is implementable without backend changes.

If this is built in V2, the effort is: 1 day for text selection + sidebar state, 1 day for "submit feedback" action integration with the existing chat/task system.

### Diff between artifact versions

Deferred. Requires artifact history (storing previous versions of each artifact). Current storage model is one file per artifact type — no versioning. Git history is the version control layer; surfacing git diff in the UI is non-trivial and not validated as a user need.

### Real-time artifact streaming during agent run

Deferred. Currently the artifact file is written at the end of the agent run. Streaming partial Markdown as the agent writes it requires server-sent events for file content (the SSE infrastructure exists, but piping file writes to SSE is new). High effort, not validated as blocking.

---

## Implementation Notes

**Sequencing:** The changes are independent. They can be built as separate PRs and shipped incrementally. Sequence: height cap → file links → teaser → agent instruction → TOC.

**Testing:** `ArtifactViewer.tsx` changes need visual review against real plan artifacts (spec, design, tasks). The height cap removal needs confirmation that no layout breaks occur in `FeatureDetail.tsx`.

**Config:** `.sdlc/config.yaml` gets a new `settings.ide_uri_scheme` field. Default `vscode`. Document in config schema comments.

**Shared component:** `MarkdownContent.tsx` is the shared rendering core for both `ArtifactViewer` (feature detail) and `WorkspacePanel` (ponder/investigation). All rendering improvements (TOC, file links) in `MarkdownContent.tsx` propagate to both surfaces automatically.
