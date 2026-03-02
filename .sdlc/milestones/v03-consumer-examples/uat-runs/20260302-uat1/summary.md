# UAT Run — Reference consumer implementations
**Date:** 2026-03-02T03:55:55Z
**Verdict:** PassWithTasks
**Tests:** 11/12 acceptance criteria pass
**Tasks created:** claude-code-consumer#T2

## Scope

No Playwright e2e spec exists for this milestone. This is a CLI/library package
(`packages/sdlc-agent`) rather than a UI feature, so UAT was conducted by direct
artifact inspection and runtime verification of the package exports and behavior.

## Results

Suite: Acceptance criteria inspection (12 criteria from spec.md)
Duration: ~2 minutes
Passed: 11 | Failed: 0 | Skipped: 0 | Gap (task created): 1

### AC1 — `sdlc-agent run <slug>` loop ✓
Runner has `while (true)` loop; CLI wires `run <slug>` command with model/maxTurns/cwd options.

### AC2 — Typed MCP tools ✓
All 7 tools present: `sdlc_get_directive`, `sdlc_write_artifact`, `sdlc_approve_artifact`,
`sdlc_reject_artifact`, `sdlc_add_task`, `sdlc_complete_task`, `sdlc_add_comment`.

### AC3 — Full directive message as prompt ✓
`runner.ts` uses `directive.message` and `directive.action` to build the Claude prompt.

### AC4 — Shell gates before `sdlc_approve_artifact` ✓
`gates.ts` runs auto shell gates via `execFile`; `approve.ts` calls `runGates` before
forwarding to `SdlcClient.approveArtifact`. Verified: echo gate passes, `false` gate fails.

### AC5 — Human gates pause execution ✓
`runner.ts` returns `{ stoppedAt: 'human_gate' }` on `isHumanGateAction`; CLI prints
"Resume after approval with: sdlc-agent run <feature>".

### AC6 — Session context persists at `.sdlc/features/<slug>/.agent-session` ✓
`session.ts` saves/loads/clears session IDs at exact path. Tested: write → load → verify
path → clear → confirm undefined.

### AC7 — Correct specialized agent per ActionType ✓
All 9 action mappings verified at runtime:
create_spec/design/tasks/qa_plan/implement_task/fix_review_issues → sonnet;
create_review/create_audit → opus; run_qa → sonnet.

### AC8 — `--model` flag overrides default ✓
CLI `run` command has `.option("--model <model>", ..., "claude-sonnet-4-6")` and wires
it through to `runFeature` options.

### AC9 — Opus agents default to `claude-opus-4-6` ✓
`reviewerAgent.model === 'claude-opus-4-6'` and `auditorAgent.model === 'claude-opus-4-6'`.

### AC10 — `run-all` discovers features via listFeatures ✓
`cli.ts` `run-all` command calls `client.listFeatures()`, filters non-released/merge phases,
and processes each serially.

### AC11 — `onMessage` callback streaming ✓
`runner.ts` passes `onMessage` through to the Claude Agent SDK `query` call.

### AC12 — REST mode (`--rest <url>`) ⚠️ TASK CREATED
CLI and `SdlcClient` implement CLI subprocess mode only; no `--rest <url>` flag or
HTTP-backed client variant exists. This matches the Open Questions note in the spec
("Server integration: Phase 5") but the criterion was still listed as AC12.
Task created: `claude-code-consumer#T2`.

## Failures

| Test | Classification | Resolution |
|---|---|---|
| AC12: REST mode `--rest <url>` not implemented | Code gap (deferred) | Task claude-code-consumer#T2 created |
