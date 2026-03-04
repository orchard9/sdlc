# Code Review: _shared/sdlc.ts — typed state access for tools

## Summary

This review covers the implementation of `.sdlc/tools/_shared/sdlc.ts` and the associated changes to `crates/sdlc-cli/src/cmd/init/commands/sdlc_tool_build.rs` and `sdlc_tool_audit.rs`.

**Overall verdict: APPROVED with one tracked issue (no blockers)**

---

## File: `.sdlc/tools/_shared/sdlc.ts`

### Correctness

**`getProjectRoot()`** — Uses `getEnv` from `runtime.ts` for cross-runtime compatibility. Falls back to `process.cwd()` with a Deno-safe guard. Correct.

**`readVision()`** — Simple defensive read. Returns `''` on any error. No issue.

**`readFeatures()`** — Correctly walks feature directories, reads manifests, and maps to `FeatureSummary`. Task parsing via `parseYamlArray` is sound. Corrupted manifests are skipped with a stderr warning using `console.error` directly (not `makeLogger`). This is acceptable for a shared module that doesn't have a tool name to log under.

**`readMilestones()`** — Correct. The `features:` parsing loop properly handles the simple string-list array format used in milestone manifests.

**`readBeat()`** — The parser (`parseBeatEvaluations`, `parseBeatWeekly`) handles the two-level nesting in `beat.yaml`. Returns `{ evaluations: [] }` on missing file.

**`writeBeat()`** — Atomic write using temp + rename. The `serializeBeat()` function covers all fields. String escaping via `yamlStr()` replaces double-quotes with single-quotes and strips newlines — this is a safe convention for the controlled schemas in `beat.yaml` (no values that contain both quote types in practice).

**`createPonder()`** — Delegates entirely to `sdlc ponder create`. Extracts slug as the last token of stdout. Throws on non-zero exit. Correct.

**`appendPonderSession()`** — Strictly follows the two-step session log protocol. Temp file cleanup is in a `finally` block so it runs even if the `spawnSync` throws. Throws on non-zero exit. Correct.

### Potential Issues

**Issue 1 (minor):** `statSync` is imported but never used in the final implementation. The import was included for potential future use but is dead code now.
- **Action:** Track as T6 to remove unused import in a cleanup pass. Not a blocker.

**Issue 2 (minor):** The `parseYamlArray` function uses `new RegExp(...)` inside a loop to build the pattern `^${key}:`. For the expected call volumes (dozens of features), this has no performance impact, but a pre-compiled regex or a string comparison would be cleaner.
- **Action:** Accept — the pattern is trivially simple and the performance is irrelevant at this scale.

**Issue 3 (advisory):** `parseBeatEvaluations` has non-trivial state machine logic (6 boolean flags and two stacks). It handles the known beat.yaml format correctly, but would silently produce wrong output on malformed YAML that partially matches the expected indentation pattern. This is acceptable for a known-schema file.
- **Action:** No change needed. The function's contract is "parses our beat.yaml format" not "parses arbitrary YAML."

### Code Style

- Consistent with the existing `_shared/` modules: no external deps, Node.js built-ins only, named exports, JSDoc on public functions.
- The file header clearly explains the contract and invariants.
- Private helpers are clearly marked as private via comments.

---

## File: `crates/sdlc-cli/src/cmd/init/commands/sdlc_tool_build.rs`

- New step 5 "Use shared modules" correctly documents all five `_shared/` modules in a table.
- `_shared/sdlc.ts` entry is accurate: lists all exported functions, explains when to use vs. shelling out.
- The `createPonder` / `appendPonderSession` carve-out note is accurate and important.
- Step numbering updated correctly (old 5–13 → new 6–14).
- Self-check list updated with the `_shared/sdlc.ts` item.
- Playbook and skill variants both updated consistently.

**No issues.**

---

## File: `crates/sdlc-cli/src/cmd/init/commands/sdlc_tool_audit.rs`

- New "State access (1 check)" section added in all three skill variants (Claude command, playbook, skill).
- Count updated from 18 to 19 consistently across description strings.
- Category breakdown updated from "5 categories" to "6 categories" in the skill variant.
- The check wording is precise: flags both raw `readFileSync` on manifest paths AND `execSync('sdlc feature list --json')`.

**No issues.**

---

## Rust Build

`SDLC_NO_NPM=1 cargo build --all` — `Finished` with 0 errors. Pre-existing warnings in `sdlc-server` are unrelated to this change.

---

## Tracked Issues

| ID | Description | Severity | Disposition |
|---|---|---|---|
| T6 | Remove unused `statSync` import from `_shared/sdlc.ts` | low | Create task, fix in next cycle |

---

## Approval Decision

The implementation satisfies all acceptance criteria from the spec. The module is defensive, follows existing conventions, uses no external dependencies, and is correctly documented in the skill templates. The one unused import is cosmetic and does not affect runtime behavior.

**APPROVED — proceed to audit.**
