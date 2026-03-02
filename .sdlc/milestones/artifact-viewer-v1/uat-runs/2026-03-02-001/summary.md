# UAT Run — Rich Artifact and Plan Viewer V1
**Date:** 2026-03-02T09:30:00Z
**Verdict:** Failed
**Tests:** 0/9 (Playwright spec generated but server start failed; static code analysis used as authoritative basis)
**Tasks created:** artifact-viewer-height-fix#T2, artifact-file-links#T7, artifact-tldr-teaser#T1, artifact-fullscreen-toc#T6

## Context

Milestone has no `acceptance_test.md` defined. Mode B was used: Playwright spec was generated from feature specs at `frontend/e2e/milestones/artifact-viewer-v1.spec.ts`. Server failed to start for automated browser run; static source code analysis (definitive) substituted.

## Code Audit Results

Static inspection of the four feature implementation targets against their specs:

| Feature | Target File | Expected Change | Status |
|---|---|---|---|
| artifact-viewer-height-fix | `ArtifactViewer.tsx:36` | Remove `max-h-96 overflow-y-auto` | **MISSING** — still present |
| artifact-file-links | `MarkdownContent.tsx` code handler | Add `FILE_PATH_PATTERN` + IDE link render | **MISSING** — no detection code |
| artifact-tldr-teaser | `ArtifactViewer.tsx` | Add `extractTeaser()`, `formatRelativeTime()`, teaser row | **MISSING** — none present |
| artifact-fullscreen-toc | `MarkdownContent.tsx`, `FullscreenModal.tsx` | `showToc` prop, heading IDs, TOC rail, `hasToc` | **MISSING** — none present |

## Failures

| Test | Classification | Resolution |
|---|---|---|
| F1: artifact card has no max-height constraint | Code bug — feature not implemented | Task artifact-viewer-height-fix#T2 created |
| F1: fullscreen button still appears | Would pass (button exists) | N/A |
| F1: fullscreen modal opens and closes | Would pass (modal exists) | N/A |
| F2: inline code file path renders as IDE link | Code bug — feature not implemented | Task artifact-file-links#T7 created |
| F3: artifact card shows teaser text + timestamp | Code bug — feature not implemented | Task artifact-tldr-teaser#T1 created |
| F3: teaser text capped at 120 chars | Code bug — feature not implemented | Task artifact-tldr-teaser#T1 created |
| F4: fullscreen modal shows sticky TOC rail | Code bug — feature not implemented | Task artifact-fullscreen-toc#T6 created |
| F4: TOC contains heading entries | Code bug — feature not implemented | Task artifact-fullscreen-toc#T6 created |
| F4: clicking TOC link scrolls to heading | Code bug — feature not implemented | Task artifact-fullscreen-toc#T6 created |

## Note on Prior QA Results

All 4 features claimed PASS in their individual `qa-results.md` files. Source inspection reveals these were false positives — the QA agents verified expected code patterns but those patterns were never actually written to disk. This is a signal that the QA runs for these features read the spec (expected state) rather than inspecting the actual working tree.
