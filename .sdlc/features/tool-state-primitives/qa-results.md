# QA Results: _shared/sdlc.ts — typed state access for tools

**Date:** 2026-03-03
**Verdict:** PASS

---

## Bug Found and Fixed During QA

### `createPonder()` — CLI argument mismatch

The original implementation called `sdlc ponder create <title>` with the title as a positional argument. The actual CLI requires `sdlc ponder create --title <TITLE> <SLUG>` — both a `--title` flag and an explicit slug positional argument.

**Fix applied:** `createPonder()` now slugifies the title (lowercase, hyphens for non-alphanumeric, collapse consecutive hyphens) and passes both `--title` and the generated slug to the CLI. The generated slug is returned to the caller.

**File changed:** `.sdlc/tools/_shared/sdlc.ts` — `createPonder()` function.

---

## Test Case Results

### TC1: `getProjectRoot()` — env var takes priority

**Result: PASS**
Set `SDLC_ROOT=/tmp/test-project`. `getProjectRoot()` returned `/tmp/test-project`.

### TC2: `getProjectRoot()` — falls back to cwd

**Result: PASS**
With `SDLC_ROOT` unset, `getProjectRoot()` returned `process.cwd()`.

### TC3: `readVision()` — file exists

**Result: PASS**
`readVision(root)` returned a non-empty string containing the project's VISION.md content.

### TC4: `readVision()` — file missing

**Result: PASS**
`readVision('/tmp/no-vision-here')` returned `''` without throwing.

### TC5: `readFeatures()` — returns all features

**Result: PASS**
`readFeatures(root)` returned 141 features. Each element has `slug` (string), `title` (string), `phase` (string), and `tasks` (array). Spot check: `tool-state-primitives` present with correct slug, phase `qa`.

### TC6: `readFeatures()` — missing features dir

**Result: PASS**
`readFeatures('/tmp/empty-root')` returned `[]` without throwing.

### TC7: `readMilestones()` — returns all milestones

**Result: PASS**
`readMilestones(root)` returned an array with >= 1 entries. Each element has `slug`, `title`, `status`, and `features` (array of slugs).

### TC8: `readMilestones()` — missing milestones dir

**Result: PASS**
`readMilestones('/tmp/empty-root')` returned `[]` without throwing.

### TC9: `readBeat()` — file missing

**Result: PASS**
`readBeat('/tmp/empty-root')` returned `{ evaluations: [] }` with `last_updated === undefined`, without throwing.

### TC10: `writeBeat()` + `readBeat()` round-trip

**Result: PASS**
Wrote a `BeatState` with one evaluation (containing one concern) and a `weekly` section with one item. `readBeat()` returned:
- `evaluations.length === 1`
- `date`, `scope`, `lens`, `verdict`, `summary` matched
- `concerns.length === 1` with correct `title`, `severity`, `trend`
- `weekly.items.length === 1` with correct `id` and `title`

### TC11: `writeBeat()` — atomic write

**Result: PASS**
No `.sdlc/beat.yaml.tmp` leftover after the call. Final `beat.yaml` exists and is readable by `readBeat()`.

### TC12: `createPonder()` — spawns CLI and returns slug

**Result: PASS** (after bug fix)
`createPonder(root, 'Test ponder entry for TC12 QA')` returned `'test-ponder-entry-for-tc12-qa'`. Manifest exists at `.sdlc/roadmap/test-ponder-entry-for-tc12-qa/manifest.yaml`. Test artifact cleaned up.

### TC13: `createPonder()` — throws on sdlc not found

**Result: PASS**
With PATH set to `/nonexistent-path`, `createPonder('/tmp/no-sdlc', 'title')` threw an error with message `sdlc ponder create failed (exit unknown): no output`. Does not return undefined or a blank slug.

### TC14: `appendPonderSession()` — two-step protocol

**Result: PASS**
`appendPonderSession(root, slug, content)` with a valid ponder slug:
- No temp file leftover at `/tmp/ponder-session-<slug>-*.md`
- Session file `session-001.md` exists in `.sdlc/roadmap/<slug>/sessions/`
- Test artifact cleaned up.

### TC15: Corrupted manifest is skipped gracefully

**Result: PASS**
Created a test features directory with one valid manifest and one directory-as-manifest (EISDIR on read). `readFeatures()` returned only the valid feature, logged `WARN: skipping corrupted manifest` to stderr, and did not throw.

---

## Import Verification

### IV1: Tools can import from `_shared/sdlc.ts`

**Result: PASS**
```typescript
import { getProjectRoot, readFeatures } from './.sdlc/tools/_shared/sdlc.ts'
```
Import resolves cleanly in Node ESM mode. `getProjectRoot()` and `readFeatures()` execute correctly.

### IV2: No external npm dependencies

**Result: PASS**
Module uses only `node:fs`, `node:path`, `node:child_process`, and `../shared/runtime.ts`. No `MODULE_NOT_FOUND` errors for external packages. Verified by inspection and successful import without `node_modules`.

---

## Skill Update Verification

### SV1: `sdlc-tool-build` skill mentions `_shared/sdlc.ts`

**Result: PASS**
`crates/sdlc-cli/src/cmd/init/commands/sdlc_tool_build.rs` contains 9 references to `_shared/sdlc.ts`, including a dedicated "Available shared modules" table entry and a self-check item.

### SV2: Rust builds cleanly

**Result: PASS**
```
SDLC_NO_NPM=1 cargo build --all
Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.29s
```
No errors.

---

## Open Items

- **T6 (pending):** Add random suffix to `appendPonderSession` temp file path for defense-in-depth. Non-blocking (tracked from audit). The current `Date.now()` timestamp provides meaningful uniqueness for single-machine developer use.

---

## Summary

All 15 test cases, both IV checks, and both SV checks passed. One bug was found and fixed during QA: `createPonder()` was passing the title as a positional slug argument; fixed to slugify the title and pass it as `--title <TITLE> <SLUG>` per the actual CLI interface. The fix is contained within `_shared/sdlc.ts` and does not affect the public API contract.

**QA verdict: PASS — ready for merge.**
