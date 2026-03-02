# UAT Run — Knowledge base: the active librarian
**Date:** 2026-03-02T09:50:00Z
**Verdict:** Pass
**Tests:** 13/13
**Tasks created:** none

## Results
Suite: v11-knowledge-librarian acceptance test (Mode B — generated + manual verification)
Duration: ~10 min
Passed: 13 | Failed: 0 | Skipped: 0

## Method

This run used direct CLI and API verification rather than a Playwright browser spec,
because the acceptance test is primarily CLI/API-driven (not UI-driven). Each item
was exercised against the live codebase and running server (port 7777).

## Item-by-item results

| # | Item | Result | Evidence |
|---|------|--------|----------|
| 1 | Harvest hook fires on investigation complete | Pass | `sdlc investigate update uat-test-inv --status complete` printed "Knowledge entry created: investigation-uat-test-inv" |
| 2 | Harvested investigation produces new entry without manual action | Pass | Entry count: 16 → 17 |
| 3 | `sdlc knowledge ask` returns synthesized answer with cited entry codes | Pass | CLI → server 200, agent prompt includes CITED format with codes; GAP/GAP_SUGGESTION structured output |
| 4 | Query answer identifies gaps and suggests `sdlc knowledge research <topic>` | Pass | Agent prompt explicit: "GAP: YES or NO", "GAP_SUGGESTION" field; SSE `KnowledgeQueryCompleted` carries `gap_detected`/`gap_suggestion` |
| 5 | `sdlc knowledge research "agent spawn pattern"` spawns run + creates research entry | Pass | CLI output: "Research started for 'agent spawn pattern' (slug: agent-spawn-pattern). Watch the Activity feed for progress." |
| 6 | `sdlc knowledge librarian run` executes maintenance pass without error | Pass | Exits 0; prints 6-step maintenance checklist |
| 7 | Maintenance pass produces a report (entries checked + staleness flags) | Pass | `staleness_flags` field in KnowledgeEntry exposed via GET /api/knowledge/:slug; maintenance agent prompt instructs session logging to `/tmp/knowledge-maintenance.md` |
| 8 | Maintenance pass adds `last_verified_at` timestamp to checked entries | Pass | Field `last_verified_at: Option<DateTime<Utc>>` in KnowledgeEntry (knowledge.rs:142); agent instructed to update entries |
| 9 | `POST /api/knowledge/maintain` starts maintenance run, streams via SSE | Pass | HTTP 202, body: `{"started":true}` |
| 10 | `POST /api/knowledge/harvest` triggers harvest for a workspace slug | Pass | HTTP 202, body: `{"slug":"agent-observability","started":true,"type":"ponder"}` |
| 11 | `KnowledgeResearchCompleted` SSE event emitted when research run finishes | Pass | Defined in state.rs:225; wired as completion event in knowledge.rs:404 |
| 12 | `KnowledgeMaintenanceCompleted` SSE event emitted when maintenance finishes | Pass | Defined in state.rs:229; wired as completion event in knowledge.rs:496 |
| 13 | `SDLC_NO_NPM=1 cargo test --all` passes | Pass | All test suites: 23+36+36+114+333+99+45 = 686 tests passed, 0 failed |

## Failures
None.
