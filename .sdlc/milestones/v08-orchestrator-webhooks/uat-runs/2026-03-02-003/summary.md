# UAT Run — Webhook ingestion and routing — external triggers fire tools on next tick
**Date:** 2026-03-02T08:21:07Z
**Run ID:** 2026-03-02-003 (run 4 overall)
**Verdict:** Pass
**Tests:** 13/13
**Tasks created:** none

## Results

Suite: milestones/v08-orchestrator-webhooks.spec.ts
Duration: 1635ms
Passed: 13 | Failed: 0 | Skipped: 0

## What changed from run 3

Run 3 (12/13) failed because the T7 fix (share `ActionDb` via `Arc<Mutex<>>` in AppState)
had been applied to source code but the installed binary at `~/.cargo/bin/sdlc` (compiled at
00:53) predated the fix. The Playwright test server on port 7777 (PID 91297, started at 12:40AM)
was using the old binary.

**Fix applied:** `cargo install --path crates/sdlc-cli --locked` rebuilt and installed the binary
with the T7 fix. The stale test server on port 7777 was killed and Playwright started a fresh
server using the new binary. All 13 tests passed.

## Failures

None.
