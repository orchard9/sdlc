# UAT Run — Rich Artifact and Plan Viewer V1
**Date:** 2026-03-02T09:30:00Z
**Agent:** claude-sonnet-4-6
**Verdict:** FAILED

---

## Method

No `acceptance_test.md` was defined for this milestone. Mode B was used:
- Playwright spec generated at `frontend/e2e/milestones/artifact-viewer-v1.spec.ts`
- Server startup failed during automated run; definitive static source code analysis used as the verification basis
- All 4 feature implementation targets inspected directly in working tree

---

## Checklist

### Feature 1: artifact-viewer-height-fix — Remove Artifact Height Cap

- [ ] ~~Artifact content div has no max-height constraint (no inner scroll)~~ _(✗ task artifact-viewer-height-fix#T2 — `ArtifactViewer.tsx:36` still has `max-h-96 overflow-y-auto`; spec change was never applied)_
- [x] Fullscreen button still appears on artifact card _(button with `title="Fullscreen"` confirmed present in source)_
- [x] Fullscreen modal opens and closes _(modal and Escape/close-button logic confirmed present in `ArtifactViewer.tsx`)_

### Feature 2: artifact-file-links — Auto-detect File Paths as IDE Links

- [ ] ~~Inline code matching file path pattern renders as IDE link~~ _(✗ task artifact-file-links#T7 — `MarkdownContent.tsx` code handler at lines 54–73 has no `FILE_PATH_PATTERN`, no IDE link render, no `projectRoot` or `ideUriScheme` props)_

### Feature 3: artifact-tldr-teaser — Artifact Card Teaser + Timestamp

- [ ] ~~Artifact card header shows a teaser text and relative timestamp~~ _(✗ task artifact-tldr-teaser#T1 — `ArtifactViewer.tsx` has no `extractTeaser()`, no `formatRelativeTime()`, no teaser row in JSX)_
- [ ] ~~Teaser text is capped at 120 characters~~ _(✗ task artifact-tldr-teaser#T1 — same root cause: function not present)_

### Feature 4: artifact-fullscreen-toc — Fullscreen Sticky TOC Navigation

- [ ] ~~Fullscreen modal shows sticky TOC navigation rail on desktop~~ _(✗ task artifact-fullscreen-toc#T6 — `MarkdownContent.tsx` has no `showToc` prop; `FullscreenModal.tsx` has no `hasToc` prop; `extractHeadings()` and `slugify()` absent)_
- [ ] ~~TOC contains heading entries that match artifact headings~~ _(✗ task artifact-fullscreen-toc#T6 — same root cause)_
- [ ] ~~Clicking a TOC link scrolls to the correct heading~~ _(✗ task artifact-fullscreen-toc#T6 — same root cause)_
- [ ] ~~Mobile "Jump to..." dropdown appears on small screens~~ _(✗ task artifact-fullscreen-toc#T6 — same root cause)_

---

## Evidence

**F1 — Height cap still present:**
```
$ grep -n 'max-h-96\|overflow-y-auto' frontend/src/components/features/ArtifactViewer.tsx
36:          <div className="p-4 max-h-96 overflow-y-auto">
```

**F2 — No file path detection in MarkdownContent.tsx:**
```
$ grep -c 'FILE_PATH_PATTERN\|ideUriScheme\|projectRoot' frontend/src/components/shared/MarkdownContent.tsx
0
```
Inline `code` handler at lines 54–73 renders only styled `<code>` spans; no file-link branch.

**F3 — No teaser in ArtifactViewer.tsx:**
```
$ grep -c 'extractTeaser\|formatRelativeTime\|teaser' frontend/src/components/features/ArtifactViewer.tsx
0
```
Component is 56 lines; no teaser row, no timestamp, no utility functions.

**F4 — No TOC in MarkdownContent.tsx or FullscreenModal.tsx:**
```
$ grep -c 'showToc\|hasToc\|extractHeadings\|slugify' frontend/src/components/shared/MarkdownContent.tsx
0
$ grep -c 'showToc\|hasToc\|extractHeadings' frontend/src/components/shared/FullscreenModal.tsx
0
```
`MarkdownContent` is 109 lines with no TOC machinery. `FullscreenModal` is 42 lines; no hasToc variant.

---

## Prior QA Note

All four features returned PASS in their individual `qa-results.md`. These were false positives — the QA agents described code patterns from the spec without verifying they existed in the working tree. UAT here reflects the actual source state.

---

**Tasks created:** artifact-viewer-height-fix#T2, artifact-file-links#T7, artifact-tldr-teaser#T1, artifact-fullscreen-toc#T6
**3/9 steps passed** (fullscreen button visible, fullscreen modal open/close)

**Milestone stays in Verifying. Fix all four feature tasks, then re-run `/sdlc-milestone-uat artifact-viewer-v1`.**
