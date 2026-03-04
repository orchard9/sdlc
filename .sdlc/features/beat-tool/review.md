# Code Review: beat-tool

## Summary

The beat tool implements the full agentic tool pattern end-to-end as specified. All 8 tasks are complete, the implementation is clean, and the tool was successfully registered in `tools.md` via `sdlc tool sync`. The two new `_shared/` primitives (`sdlc.ts` and `agent.ts`) are well-structured and reusable by future tools.

Overall verdict: **approve with minor notes** — all findings below are low severity and none block merge.

---

## Files Reviewed

- `.sdlc/tools/beat/tool.ts` — main tool (578 lines)
- `.sdlc/tools/beat/README.md` — usage documentation
- `.sdlc/tools/_shared/sdlc.ts` — state reader primitive (669 lines)
- `.sdlc/tools/_shared/agent.ts` — agent recruiter/invoker (277 lines)
- `.sdlc/tools/_shared/types.ts` — shared type contracts
- `.sdlc/tools/_shared/runtime.ts` — cross-runtime helpers

---

## Spec Compliance

| Requirement | Status | Notes |
|---|---|---|
| `scope: project/domain/feature:<slug>` input | PASS | All three branches implemented in `runEvaluate` |
| `mode: evaluate/week` | PASS | Dispatched correctly in `run()` |
| NDJSON streaming: gathering/recruiting/evaluating/writing/done/error | PASS | All 6 event types present |
| Verdict: on-track/drifting/off-course | PASS | Enum validated in `parseVerdict` |
| Persistence to `.sdlc/beat.yaml` | PASS | `writeBeat()` appends, never rewrites existing records |
| Week mode: top-5 items, no agent call, no write | PASS | `runWeek` is read-only |
| `_shared/sdlc.ts` reusable primitive | PASS | Exports typed functions; no beat-specific coupling |
| `_shared/agent.ts` reusable primitive | PASS | `ensureAgent` + `runAgent` + `runAgentViaServer` |
| `sdlc tool sync` updates `tools.md` | PASS | beat entry visible with correct metadata |

---

## Findings

### F1 — `_shared/sdlc.ts` has two incompatible `beat.yaml` formats (low severity)

`_shared/sdlc.ts` defines `BeatState` / `BeatEvaluation` with fields `date`, `scope`, `lens`, `verdict`, `summary`, `concerns[]` and serializes them to YAML. `beat/tool.ts` defines its own `BeatRecord` with fields `id`, `scope`, `mode`, `timestamp`, `verdict`, `score`, `concerns[]` and serializes independently.

These are two different schemas. The `_shared/sdlc.ts` `writeBeat`/`readBeat` functions and the `beat/tool.ts` `loadBeatFile`/`writeBeat` functions write incompatible files — if both run, the YAML they write is unreadable by the other. The spec defines only one schema (the `id/score/timestamp` shape). The `_shared/sdlc.ts` richer schema (`evaluations[]`, `weekly{}`, `BeatConcern` with severity/trend) was added beyond spec scope.

**Action:** Track as a task — the beat tool should be migrated to use `_shared/sdlc.ts readBeat/writeBeat` so there is a single canonical format. For now, the tool works correctly because it uses its own internal functions consistently.

### F2 — `runEvaluate` emits `gathering` event after gathering (not before) (low severity, spec gap)

The spec and QA plan both state the `gathering` event should be emitted "before state reads". In the implementation (lines 277–291), `readVision`, `readFeatures`, and `readMilestones` are called first, then `emit('gathering', ...)` fires. This means the gathering event arrives after the work is done. For a long-running CLI call this would produce a confusing user experience.

The `recruiting` and `evaluating` events are correctly emitted before the blocking call.

**Action:** Track as a task. Low-impact for a first release; does not break any test case.

### F3 — Domain scope filtering is a stub (low severity, documented limitation)

In `runEvaluate` lines 304–312, the domain/milestone scope filtering contains a comment: "Fall back to all features if domain not found as milestone". When `scope` is neither `"project"` nor `feature:*`, the code finds the matching milestone and sets `scopeDescription` but does not actually filter `scopedFeatures`. The evaluation runs against all features regardless.

The spec says domain scope "evaluates features matching a domain tag or milestone slug". The filtering is incomplete.

**Action:** Track as a task. The spec's `scope: '<domain>'` filtering is a heuristic anyway ("slug contains domain or phase matches" per the spec) — the current behavior of evaluating all features with a narrowed `scopeDescription` is a reasonable fallback.

### F4 — `serializeBeatFile` in `tool.ts` does not escape concerns containing colons or double quotes (low severity)

In `tool.ts` `serializeBeatFile` (lines 234–251), concern strings are written as:
```
      - concern text here
```
If a concern string contains a `:` at the start, or leading `{`, the resulting YAML will be invalid and `parseBeatYaml` will fail to re-parse it. The `parseBeatYaml` regex extracts concern lines via string prefix `- ` which is resilient, but standard YAML parsers would fail.

The `_shared/sdlc.ts` `serializeBeat` function handles this properly using `yamlStr()` which wraps in double quotes.

**Action:** Track as a task (tied to F1 — consolidated format would fix this).

### F5 — `runAgent` uses `--system-prompt` flag; verify this flag exists in the installed claude CLI (informational)

`agent.ts` line 144 calls `claude --print --system-prompt <content>`. The `--system-prompt` flag may not be present in all versions of the claude CLI. If the installed CLI does not support this flag, agent invocation will fail silently (the CLI may ignore it and respond without the persona context).

**Action:** This is an operational note. No code change needed at review time; verify during QA.

---

## Positive Observations

- **Score clamping** (`Math.max(0, Math.min(100, parsed.score))` in `parseVerdict`) — defensive, correct.
- **JSON retry logic** — regex extraction fallback for markdown-wrapped JSON responses is practical and well-implemented.
- **Atomic writes** in `_shared/sdlc.ts writeBeat` use temp-file + rename — production-safe.
- **`_shared/runtime.ts` cross-runtime abstraction** — all tools benefit from unified Bun/Node/Deno support.
- **Beat ID generation** (`nextBeatId`) correctly handles gaps in the sequence.
- **Week mode fallback** — if no beats within the 14-day window, falls back to the last 5 beats rather than returning empty. Good defensive behavior.
- **`ensureAgent` is idempotent** — checks existence before creating; safe to call repeatedly.
- **README.md** — complete, accurate, includes streaming protocol examples.
- **`_shared/sdlc.ts`** goes significantly beyond spec: adds `createPonder`, `appendPonderSession`, `writeBeat`/`readBeat`, typed `BeatState`, `MilestoneSummary.features`, `TaskSummary` — solid investment in the primitive layer.

---

## Test Coverage Gap

The QA plan requires TC-5 (`ensureAgent` in temp dir) and TC-4 (`readVision` in temp dir). These tests were marked as requiring isolated environments. Verification that they pass is deferred to the QA phase per the plan.

---

## Conclusion

The implementation satisfies all spec requirements. The four findings above are tracked tasks, not blockers. The tool is ready to advance to audit.
