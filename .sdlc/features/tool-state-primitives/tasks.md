# Tasks: _shared/sdlc.ts — typed state access for tools

## T1: Write `_shared/sdlc.ts` with `readVision()`, `readFeatures()`, `readMilestones()`

Create `/Users/jordanwashburn/Workspace/orchard9/sdlc/.sdlc/tools/_shared/sdlc.ts`.

Implement the module skeleton with:
- Private path helpers (`featuresDir`, `milestonesDir`, `visionPath`)
- `parseYamlScalars()` — flat key:value YAML parser
- `parseYamlArray()` — parse named array blocks from YAML  
- `parseManifest()` — top-level manifest parser (scalars + arrays)
- `getProjectRoot()` — resolves `SDLC_ROOT` env var or `process.cwd()`
- `readVision(root)` — reads `VISION.md`, returns `''` if missing
- `readFeatures(root)` — walks `.sdlc/features/*/manifest.yaml`, returns `FeatureSummary[]`
- `readMilestones(root)` — walks `.sdlc/milestones/*/manifest.yaml`, returns `MilestoneSummary[]`
- All exported types: `FeatureSummary`, `TaskSummary`, `MilestoneSummary`

Uses only Node.js built-ins (`node:fs`, `node:path`) and `getEnv` from `runtime.ts`. No external dependencies.

---

## T2: Add `readBeat()` and `writeBeat()` to `_shared/sdlc.ts`

Extend the module with beat state read/write:

- `BeatState`, `BeatEvaluation`, `BeatConcern`, `BeatWeeklyItem` type exports
- `readBeat(root)` — reads `.sdlc/beat.yaml`, returns `{ evaluations: [] }` if missing
- `writeBeat(root, state)` — serializes `BeatState` to YAML string via `serializeBeat()`, writes atomically (temp file + rename)

The `serializeBeat()` function must produce YAML that round-trips through `readBeat()` — the two functions form a closed read/write pair. All string values must be safely quoted (escape double quotes, wrap in double-quote delimiters). Multi-line concern titles must be single-line (newlines replaced with space).

---

## T3: Add `createPonder()` and `appendPonderSession()` to `_shared/sdlc.ts`

Extend the module with ponder write operations:

- `createPonder(root, title)` — spawns `sdlc ponder create "<title>"`, extracts slug from stdout, throws on failure
- `appendPonderSession(root, slug, content)` — two-step temp file protocol:
  1. Write content to `/tmp/ponder-session-<slug>-<Date.now()>.md`
  2. Spawn `sdlc ponder session log <slug> --file <tmp>`
  3. Cleanup temp file (best-effort)
  4. Throw if command exits non-zero

Uses `spawnSync` from `node:child_process`. Respects the session logging invariant documented in MEMORY.md.

---

## T4: Export `getProjectRoot()` from `_shared/sdlc.ts`

Ensure `getProjectRoot()` is exported and uses `getEnv` from `../\_shared/runtime.ts` (not `process.env` directly) for cross-runtime compatibility with Bun and Deno.

Verify the import path works when called from a tool in `.sdlc/tools/<name>/tool.ts`. The relative import is `'../_shared/sdlc.ts'`.

This task is a verification/refinement step — `getProjectRoot` is scaffolded in T1 but this task confirms the cross-runtime path and adds a brief JSDoc comment explaining usage.

---

## T5: Update `sdlc-tool-build` and `sdlc-tool-audit` skills in `init.rs` to document `_shared/sdlc.ts` usage

File: `crates/sdlc-cli/src/cmd/init/commands/` — find the `sdlc_tool_build` and `sdlc_tool_audit` skill definitions.

**sdlc-tool-build skill:** Add `_shared/sdlc.ts` to the "Available shared modules" section with description: typed state access for `.sdlc/` files.

**sdlc-tool-audit skill:** Add to the checklist: flag tools that shell out to `sdlc feature list --json` or `sdlc milestone list --json` for simple reads — recommend `_shared/sdlc.ts` instead.

Verify with `cargo build --all` (or `SDLC_NO_NPM=1 cargo build --all`) that the changes compile.
