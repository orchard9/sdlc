# QA Plan: beat-tool

## Scope

Verify that the beat tool implements the full agentic tool pattern end-to-end: state gathering, agent recruitment/invocation, NDJSON streaming, persistence, and week mode.

## Test Cases

### TC-1: Meta output is valid

**Command:** `bun run .sdlc/tools/beat/tool.ts --meta`
**Expected:**
- Exit code 0
- Stdout is valid JSON matching `ToolMeta` shape
- `name === "beat"`
- `input_schema` includes `scope` and `mode` fields
- `requires_setup === false`

### TC-2: Shared sdlc.ts readVision returns project vision

**Method:** Import `readVision` from `_shared/sdlc.ts` in a quick test script, call with project root
**Expected:**
- Returns non-empty string containing "sdlc" or "governance"
- Does not throw when VISION.md exists

### TC-3: Shared sdlc.ts readFeatures returns feature list

**Method:** Import `readFeatures` from `_shared/sdlc.ts`, call with project root
**Expected:**
- Returns non-empty array of Feature objects
- Each item has `slug` and `phase` fields
- Does not throw on a populated project

### TC-4: Shared sdlc.ts graceful failure

**Method:** Call `readVision` with a temp directory that has no VISION.md
**Expected:** Returns empty string (no throw)

### TC-5: Shared agent.ts ensureAgent creates agent file if missing

**Method:** Call `ensureAgent(root, 'cto-cpo-lens', 'Strategic CTO/CPO...')` in a temp project
**Expected:**
- If `.claude/agents/cto-cpo-lens.md` doesn't exist, it gets created
- Returns absolute path to the agent file
- Path exists on disk after call

### TC-6: evaluate mode completes without error

**Command:** `echo '{"scope":"project","mode":"evaluate"}' | bun run .sdlc/tools/beat/tool.ts --run`
**Expected:**
- Exits 0
- Stdout contains multiple NDJSON lines
- First line has `event: "gathering"`
- Last line has `event: "done"` with `result.ok === true`
- `result.data.verdict` is one of `on-track | drifting | off-course`
- `result.data.concerns` is an array of strings
- `result.data.beat_id` is a string like `beat-001`

### TC-7: beat.yaml is written after evaluate

**After TC-6:**
**Expected:**
- `.sdlc/beat.yaml` exists
- Contains a `beats` array with at least one entry
- Entry has `id`, `scope`, `mode`, `timestamp`, `verdict`, `score`, `concerns` fields

### TC-8: evaluate mode streams all required events

**Method:** Run evaluate mode, collect all NDJSON lines
**Expected:**
- At least one `gathering` event with a `message` string
- At least one `recruiting` event with a `message` string
- At least one `evaluating` event with a `message` string
- At least one `writing` event with a `message` string
- Exactly one `done` event as the last line

### TC-9: week mode returns top-5 items without writing

**Prerequisites:** beat.yaml has at least one entry (run TC-6 first)
**Command:** `echo '{"scope":"project","mode":"week"}' | bun run .sdlc/tools/beat/tool.ts --run`
**Expected:**
- Exits 0
- Last line `result.ok === true`
- `result.data.week_items` is array of length 1–5
- Each item has `priority` (1–5) and `item` (non-empty string)
- beat.yaml is NOT modified (no new entry added)

### TC-10: feature-scoped evaluate

**Command:** `echo '{"scope":"feature:beat-tool","mode":"evaluate"}' | bun run .sdlc/tools/beat/tool.ts --run`
**Expected:**
- Exits 0
- `result.ok === true`
- `result.data.verdict` is set
- `result.data.beat_id` shows the new record was written

### TC-11: sdlc tool sync picks up beat tool

**Command:** `sdlc tool sync`
**Expected:**
- Exit 0
- `tools.md` updated to include a `beat` entry
- Entry shows correct name, description, and run command

### TC-12: Invalid input returns error result

**Command:** `echo '{"scope":"project","mode":"invalid_mode"}' | bun run .sdlc/tools/beat/tool.ts --run`
**Expected:**
- Exits 1
- Stdout is valid JSON with `ok: false`
- `error` field describes the invalid input

### TC-13: NDJSON lines are individually parseable

**Method:** Parse each stdout line from a successful evaluate run independently
**Expected:** Every line is valid JSON (no broken NDJSON)

## Acceptance Criteria

All of TC-1 through TC-11 must pass. TC-12 and TC-13 are defensive tests that should also pass.

## Testing Approach

Manual execution using `bun run` or `node` on the tool file directly. The SDLC tool runner (`sdlc tool run beat`) can be used for TC-6, TC-9, TC-10 after TC-11 (sync) passes.

All tests should be run in the actual project root (not a temp dir), except TC-4 and TC-5 which require isolated environments.
