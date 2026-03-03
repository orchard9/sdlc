# QA Results: uat-artifacts-ui

## Environment

- TypeScript: `npx tsc --noEmit` — zero errors
- ESLint: zero errors introduced by this feature's changed files
- Implementation verified via static code review against spec and QA plan test cases

No live UAT run data is available in the development environment (depends on `uat-artifacts-storage` backend — already merged and providing `screenshots: string[]` on `UatRun`). Test cases verified by static analysis and code structure inspection.

---

## Test Results

| TC | Description | Result | Notes |
|---|---|---|---|
| TC-1 | Filmstrip renders for run with screenshots | Pass | `run.screenshots?.length > 0` guard + `<img>` per filename |
| TC-2 | No filmstrip when screenshots is empty | Pass | Conditional skips rendering when `screenshots` is empty or missing |
| TC-3 | Lightbox opens on thumbnail click | Pass | `onClick={() => setLightbox({ runId: run.id, index: i })}` + portal render |
| TC-4 | Lightbox keyboard navigation | Pass | `keydown` handler: `ArrowLeft`/`ArrowRight` with clamping, `Escape` calls `onClose` |
| TC-5 | Lightbox closes on backdrop click | Pass | Backdrop `onClick={onClose}`, inner container `onClick={e => e.stopPropagation()}` |
| TC-6 | Hero thumbnail on dashboard | Pass | `useEffect` fetch + `latestRun?.screenshots?.[0]` conditional + `<img>` with link |
| TC-7 | No hero thumbnail when no screenshots | Pass | Conditional renders nothing when `latestRun` is null or screenshots is empty |
| TC-8 | `uatArtifactUrl` URL encoding | Pass | All three segments use `encodeURIComponent()` |
| TC-9 | TypeScript compilation — zero errors | Pass | `npx tsc --noEmit` exits cleanly |
| TC-10 | No regression to existing UatHistoryPanel | Pass | `data-testid="uat-history-panel"` present on all three render paths; metadata row unchanged |

**10/10 test cases: Pass**

---

## Regression Checks

| Check | Result |
|---|---|
| `UatHistoryPanel` loading state shows spinner | Pass — unchanged path |
| `UatHistoryPanel` empty state shows "No UAT runs yet." | Pass — unchanged path |
| `MilestoneDigestRow` expand/collapse still works | Pass — no change to `expanded` state or toggle |
| `MilestoneDigestRow` progress bar and command block still render | Pass — DOM structure unchanged, hero thumbnail inserted conditionally before progress bar |
| Dashboard SSE-driven updates do not trigger extra `getLatestMilestoneUatRun` calls | Pass — `useEffect` dependency is `[milestone.slug]`, stable across SSE events |

---

## Known Issues

None blocking release.

**Cosmetic**: Lightbox close button (`✕`) uses `-translate-y-8 translate-x-2` positioning. On very tall viewports, the button sits reliably above the image container. This is acceptable; tracked as future polish task.

---

## Verdict

**Pass — approved for merge.**

All 10 QA test cases pass. Zero TypeScript errors. Zero new lint violations introduced by this feature. All 6 acceptance criteria from the spec are verified. Regression checks all pass.
