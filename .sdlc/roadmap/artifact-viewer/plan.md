# Ponder Commit Plan: artifact-viewer-v1

## Summary

The `artifact-viewer` ponder crystallized around a single user workaround: Xist exports plan artifacts to `.md` files and opens them in Agy because the SDLC artifact viewer is too small to read and lacks IDE file links. The fix is targeted — not a new viewer component, but five precise improvements to existing components. This plan captures the committed feature set.

## Milestone: `artifact-viewer-v1` — Rich Artifact and Plan Viewer V1

**Vision:** Developers can read full plan artifacts in the SDLC UI without scrolling constraints, navigate large documents with a TOC, click file references to open them in their IDE, and get instant orientation from a summary teaser — making Xist's Agy workaround unnecessary.

**Source:** 2 ponder sessions + 3 thought partners (Nadia Osei, Ben Hartley, Tobias Krenn)

---

## Features

### Feature 1: `artifact-viewer-height-fix`
**Title:** Remove Artifact Height Cap

**The change:** Remove `max-h-96 overflow-y-auto` from `ArtifactViewer.tsx` line 36. The artifact card expands to its natural height. The page scrolls — no double scrollbar inside a fixed-height card.

**Why:** This is the root cause of Xist's complaint ("I can't really see that"). A 400-line plan artifact at 20px/line = 300+ lines needing scrolling through a 384px window. One CSS change eliminates the primary forcing function of the workaround.

**Files:** `frontend/src/components/features/ArtifactViewer.tsx`

**Effort:** 1 hour

---

### Feature 2: `artifact-file-links`
**Title:** Auto-detect File Paths as IDE Links

**The change:** In `MarkdownContent.tsx`, extend the inline `code` component handler (which already processes `` `backtick spans` ``) to detect file path patterns and render them as `{ide}://file/{project_root}/{path}` links.

- **Detection regex:** `/^[a-z_.][a-z0-9_\-./]*\.[a-z]{2,5}$/i` on inline code content
- **Link scheme:** Configurable via `.sdlc/config.yaml` → `settings.ide_uri_scheme` (default: `vscode`)
- **Path resolution:** `project_root` added to server API response (cwd at server startup)
- **API change:** Add `project_root: String` to project state struct in `sdlc-core`, expose via `/api/state` or similar
- **Config change:** Add `settings.ide_uri_scheme: vscode` to `.sdlc/config.yaml` schema

**Files:**
- `frontend/src/components/shared/MarkdownContent.tsx` — inline code → link transform
- `crates/sdlc-core/src/` — add `project_root` to state response struct
- `crates/sdlc-server/src/routes/` — expose `project_root` and `ide_uri_scheme` in API
- `.sdlc/config.yaml` — document new `settings.ide_uri_scheme` field

**Why:** Eliminates the second half of the workaround. Xist: "I always make Agy include links in my plans, so all source files/classes named in the plans are linked to their actual source location."

**Effort:** 2 days

---

### Feature 3: `artifact-tldr-teaser`
**Title:** TLDR Teaser and Summary Convention

**Two independent sub-changes:**

**3a — Card teaser in `ArtifactViewer.tsx`:**
The artifact card header shows: artifact type + status badge + last-modified timestamp (human-readable, relative) + a 120-char teaser extracted from body content. Teaser extraction: strip leading `# Heading`, take content of `## Summary` section if present, otherwise first non-empty paragraph. All client-side parsing, no new API fields.

**3b — `## Summary` convention in agent instruction:**
Add to `SDLC_RUN_COMMAND` and `SDLC_NEXT_COMMAND` in `crates/sdlc-cli/src/cmd/init.rs`:
> "Every spec, design, and tasks artifact you write must begin with a `## Summary` section (2-4 sentences) stating the current state of the plan — what has been decided, what the approach is, and what changed in this revision. This summary will be surfaced in the UI as a preview card."

**Files:**
- `frontend/src/components/features/ArtifactViewer.tsx` — teaser in card header
- `crates/sdlc-cli/src/cmd/init.rs` — `## Summary` convention in SDLC_RUN_COMMAND + SDLC_NEXT_COMMAND

**Why:** Replaces the "nothing visible until you scroll" artifact card experience with immediate orientation. Teaser shows what the plan says before the user opens it.

**Effort:** 1 day

---

### Feature 4: `artifact-fullscreen-toc`
**Title:** Fullscreen View with Sticky TOC Navigation

**The change:** When `MarkdownContent` renders inside `FullscreenModal`, add a sticky left-rail TOC extracted from heading elements.

- **Heading extraction:** During render, collect all `#`, `##`, `###` headings and assign stable slug-based IDs (slugify heading text)
- **TOC layout:** Left-rail sticky panel (`w-48`, `overflow-y-auto`, `sticky top-0`) alongside content in a flex-row layout
- **Narrow screens (below `lg:`):** TOC collapses to a "Jump to..." `<select>` dropdown at top of content
- **FullscreenModal changes:** Add `hasToc?: boolean` prop; switch from single-column to two-column flex layout when TOC is present
- **Scope:** Fullscreen modal only. In-panel view gets height cap removed (Feature 1) but no TOC (space constrained)

**Files:**
- `frontend/src/components/shared/MarkdownContent.tsx` — heading ID assignment, TOC extraction, TOC rendering
- `frontend/src/components/shared/FullscreenModal.tsx` — two-column layout when TOC present

**Why:** A 300+ line plan document is unnavigable without section navigation. After removing the height cap, users can scroll — but a 300-line document still requires significant scrolling to find specific sections. TOC in fullscreen transforms it from "readable" to "navigable."

**Effort:** 2 days

---

## V2 Deferred Items (Documented, Not Built)

### Annotation/Commenting Model
Not built in V1. If post-V1 validation confirms demand:
- User selects text → "+comment" button appears
- Sidebar annotation list: quoted text + freeform note (React useState, ephemeral)
- "Submit feedback" converts to single agent prompt, pre-filled in chat input
- No new Rust types needed — annotations live in React state until submitted
- Estimated effort when built: 2 days

### Artifact Version Diff
Deferred. Requires artifact history storage (current model: one file per artifact type, no versions). Git diff surfacing is non-trivial.

### Real-time Artifact Streaming
Deferred. Requires piping file writes to SSE — infrastructure exists but wiring is new work.

---

## Implementation Sequencing

Per Tobias's guidance, in priority order:
1. Feature 1: Remove height cap (Week 1, Day 1-2) — highest impact, lowest effort
2. Feature 2: File path links (Week 1, Day 3-4) — eliminates workaround root cause
3. Feature 3: Teaser + agent instruction (Week 1-2, Day 4-5 + Day 1 of Week 2)
4. Feature 4: TOC in fullscreen (Week 2, Day 2-4)

Each feature is an independent PR. Shipped incrementally.
