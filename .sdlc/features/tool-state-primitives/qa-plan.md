# QA Plan: _shared/sdlc.ts — typed state access for tools

## Scope

Verify that `_shared/sdlc.ts` is correctly implemented according to the spec and design. All testing is manual/inspection-based since the tool layer has no test runner — verification is done by exercising the module from a tool context or Node.js REPL.

## Test Cases

### TC1: `getProjectRoot()` — env var takes priority

**Setup:** Set `SDLC_ROOT=/tmp/test-project` in environment.  
**Action:** Call `getProjectRoot()`.  
**Expected:** Returns `/tmp/test-project`.  
**Cleanup:** Unset `SDLC_ROOT`.

### TC2: `getProjectRoot()` — falls back to cwd

**Setup:** Ensure `SDLC_ROOT` is unset. cwd = `/Users/jordanwashburn/Workspace/orchard9/sdlc`.  
**Action:** Call `getProjectRoot()`.  
**Expected:** Returns the current working directory path.

### TC3: `readVision()` — file exists

**Setup:** `VISION.md` exists in the project root.  
**Action:** Call `readVision(root)` where `root` is the sdlc project root.  
**Expected:** Returns a non-empty string containing the vision content.

### TC4: `readVision()` — file missing

**Setup:** Pass a `root` path that has no `VISION.md`.  
**Action:** Call `readVision('/tmp/no-vision-here')`.  
**Expected:** Returns `''` without throwing.

### TC5: `readFeatures()` — returns all features

**Setup:** Use the sdlc project root which has many features in `.sdlc/features/`.  
**Action:** Call `readFeatures(root)`.  
**Expected:**
- Returns an array with length >= 1.
- Each element has `slug` (string, non-empty), `title` (string), `phase` (string).
- `slug` matches the directory name of the feature.
- `tasks` is an array (may be empty) where each task has `id`, `title`, `status`.

### TC6: `readFeatures()` — missing features dir

**Setup:** Pass a `root` with no `.sdlc/features/` directory.  
**Action:** Call `readFeatures('/tmp/empty-root')`.  
**Expected:** Returns `[]` without throwing.

### TC7: `readMilestones()` — returns all milestones

**Setup:** Use the sdlc project root which has milestones in `.sdlc/milestones/`.  
**Action:** Call `readMilestones(root)`.  
**Expected:**
- Returns an array with length >= 1.
- Each element has `slug`, `title`, `status`.
- `features` is an array of slugs (strings).

### TC8: `readMilestones()` — missing milestones dir

**Setup:** Pass a `root` with no `.sdlc/milestones/` directory.  
**Action:** Call `readMilestones('/tmp/empty-root')`.  
**Expected:** Returns `[]` without throwing.

### TC9: `readBeat()` — file missing

**Setup:** Pass a `root` that has no `.sdlc/beat.yaml`.  
**Action:** Call `readBeat(root)`.  
**Expected:** Returns `{ evaluations: [] }` without throwing. `last_updated` is undefined.

### TC10: `writeBeat()` + `readBeat()` round-trip

**Setup:** Use a temp directory as `root`. Create `.sdlc/` subdirectory.  
**Action:**
1. Create a `BeatState` with one evaluation containing one concern, and a `weekly` section with one item.
2. Call `writeBeat(root, state)`.
3. Verify `.sdlc/beat.yaml` exists.
4. Call `readBeat(root)`.
**Expected:**
- `evaluations` has length 1.
- The evaluation's `date`, `scope`, `lens`, `verdict`, `summary` match what was written.
- `concerns` has length 1 with correct `title`, `severity`, `trend`.
- `weekly.items` has length 1 with the correct `id` and `title`.

### TC11: `writeBeat()` — atomic write (temp file replaced)

**Setup:** Use a temp directory. Prepare initial `beat.yaml` content.  
**Action:** Call `writeBeat()`.  
**Expected:** No `.sdlc/beat.yaml.tmp` leftover after the call. The final file is valid YAML readable by `readBeat()`.

### TC12: `createPonder()` — spawns CLI and returns slug

**Setup:** Running in the sdlc project root where `sdlc` binary is on PATH.  
**Action:** Call `createPonder(root, 'Test ponder entry for TC12')`.  
**Expected:**
- Returns a non-empty slug string (e.g. `test-ponder-entry-for-tc12`).
- A directory exists at `.sdlc/roadmap/<slug>/manifest.yaml`.
- **Cleanup:** Remove the created ponder directory (or accept it as a transient test artifact).

### TC13: `createPonder()` — throws on sdlc not found

**Setup:** Temporarily break the PATH so `sdlc` is not found (or pass a non-existent root).  
**Action:** Call `createPonder('/tmp/no-sdlc', 'title')`.  
**Expected:** Throws an error with a descriptive message. Does not return undefined or a blank slug.

### TC14: `appendPonderSession()` — two-step protocol

**Setup:** Create a ponder entry (via `sdlc ponder create`) to get a valid slug.  
**Action:** Call `appendPonderSession(root, slug, '## Test session\n\nContent here.')`.  
**Expected:**
- No temp file leftover at `/tmp/ponder-session-<slug>-*.md`.
- A session file exists in `.sdlc/roadmap/<slug>/sessions/`.
- The session file contains the provided content.
- **Cleanup:** Remove the test ponder entry.

### TC15: Corrupted manifest is skipped gracefully

**Setup:** Create a test features directory with one valid manifest and one corrupted manifest (invalid YAML with truncated content).  
**Action:** Call `readFeatures(root)`.  
**Expected:** Returns one valid `FeatureSummary`. The corrupted manifest is skipped. A warning is logged to stderr.

## Import Verification

### IV1: Tools can import from `_shared/sdlc.ts`

Verify the import works from a tool file:
```typescript
import { getProjectRoot, readFeatures } from '../_shared/sdlc.ts'
```
Run with `node --input-type=module` or via `sdlc tool run` to confirm no import errors.

### IV2: No external npm dependencies

Run `node -e "require('./.sdlc/tools/_shared/sdlc.ts')"` (or equivalent for ESM) and verify no `MODULE_NOT_FOUND` errors beyond Node.js built-ins.

## Skill Update Verification

### SV1: `sdlc-tool-build` skill mentions `_shared/sdlc.ts`

After T5 is implemented: run `sdlc init --dry-run` or inspect the installed `~/.claude/commands/sdlc-tool-build.md` to confirm `_shared/sdlc.ts` appears in the "Available shared modules" section.

### SV2: Rust builds cleanly

After T5 is implemented:
```bash
SDLC_NO_NPM=1 cargo build --all 2>&1 | tail -5
```
Expected: `Finished` with no errors.

## Pass Criteria

All 15 test cases and both IV checks must pass. SV1 and SV2 must pass for T5. A single test failure blocks QA approval.
