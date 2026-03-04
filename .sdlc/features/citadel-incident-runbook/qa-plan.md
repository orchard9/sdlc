# QA Plan: Create Incident from Citadel Logs Runbook

## Scope

Validate that the `citadel_create_incident_from_logs` Pantheon runbook executes all four steps correctly, handles failures with the specified failure modes, produces properly-formatted Discord responses, and is discoverable via the Pantheon runbooks API.

---

## Test Environment

- Pantheon running locally or in staging with mock/stubbed Citadel API responses
- Mock `ToolExecutor` or test doubles for `citadel_query_logs` and `citadel_annotate_log`
- Mock `IncidentService` returning deterministic incident IDs
- Test suite: Go (`go test ./internal/runbooks/...`)

---

## Unit Test Cases

### `selectKeyEntries`

| ID | Input | Expected Output |
|---|---|---|
| UE-01 | Empty log list, maxN=5 | Empty slice |
| UE-02 | 3 error entries, maxN=5 | All 3 error entries |
| UE-03 | 3 error + 4 warn, maxN=5 | 3 errors + 2 warns |
| UE-04 | 10 info entries, maxN=5 | First entry + last entry (2 total; cap at min(maxN, available)) |
| UE-05 | 2 error + 3 warn + 5 info, maxN=5 | 2 errors + 3 warns |
| UE-06 | 1 error + 0 warn + 10 info, maxN=5 | 1 error + first info + last info (3 total) |

### `synthesizeFindings`

| ID | Input | Expected |
|---|---|---|
| SF-01 | Empty logs | Empty string or "No findings" message |
| SF-02 | 3 logs, all unique messages | Markdown bullet list with 3 items |
| SF-03 | 8 logs, 3 distinct messages (duplicates) | Bullet list with 3 unique items (capped at 5) |
| SF-04 | 10 logs, 10 distinct messages | Bullet list with 5 items (capped) |

### `buildSummary`

| ID | Scenario | Expected |
|---|---|---|
| BS-01 | Episode ID present in first log | Summary contains `episode_id` |
| BS-02 | No episode ID | Summary contains "none" for episode link |
| BS-03 | No annotation IDs | Summary contains "none" for annotations |
| BS-04 | Multiple annotation IDs | Summary lists all annotation IDs comma-separated |

### `stepQueryLogs`

| ID | Mock behavior | Expected |
|---|---|---|
| SQ-01 | Executor returns 42 logs | `ctx.Logs` has 42 entries; `ctx.LogCount == 42`; returns nil |
| SQ-02 | Executor returns error | Returns non-nil error; `ctx.Logs` is empty |
| SQ-03 | Executor returns 0 logs (ok but empty) | Returns non-nil error with "no logs found" message |
| SQ-04 | Executor returns `metadata.executed_query` | `ctx.ExecutedQuery` is set correctly |

### `stepAnnotateLogs`

| ID | Mock behavior | Expected |
|---|---|---|
| SA-01 | All 5 annotations succeed | `ctx.AnnotationIDs` has 5 entries; returns nil |
| SA-02 | 2 of 5 fail | `ctx.AnnotationIDs` has 3 entries; returns nil (non-fatal) |
| SA-03 | All 5 fail | `ctx.AnnotationIDs` is empty; returns nil |
| SA-04 | Fewer than 5 logs in ctx | Annotates only available entries; no panic |

### `stepCreateIncident`

| ID | Mock behavior | Expected |
|---|---|---|
| SC-01 | IncidentService succeeds | `ctx.IncidentID` and `ctx.IncidentURL` set; returns nil |
| SC-02 | IncidentService returns error | Returns non-nil error (halts runbook) |
| SC-03 | Severity override "critical" | Created incident has severity `critical` |
| SC-04 | Default severity | Created incident has severity `high` |
| SC-05 | `AlertSource` field | Created incident has `alert_source = "citadel"` |

### `stepUpdateAnnotations`

| ID | Mock behavior | Expected |
|---|---|---|
| SU-01 | All updates succeed | All annotations updated with real incident ID; returns nil |
| SU-02 | Some updates fail | Logs warnings; returns nil (non-fatal) |
| SU-03 | No annotation IDs in ctx | No calls made; returns nil |

---

## Integration Test Cases

### Happy path

| ID | Steps | Expected |
|---|---|---|
| IT-01 | Full runbook: query returns logs → 5 annotated → incident created → annotations updated | Incident created with correct fields; 5 annotation IDs in summary; `{PENDING}` replaced in Step 3b |
| IT-02 | Runbook discoverable | `GET /api/v1/runbooks` response includes `citadel_create_incident_from_logs` with correct name and description |

### Halt conditions

| ID | Failure point | Expected |
|---|---|---|
| IT-03 | Step 1 fails (Citadel API error) | Runbook halts; no annotation or incident calls made; Discord reports error |
| IT-04 | Step 1 returns empty logs | Runbook halts with "no logs found" message; no incident created |
| IT-05 | Step 3 (create incident) fails | Runbook halts after Step 2; Step 3b not executed; Discord reports error |

### Warn-continue conditions

| IT-06 | Step 2: all annotations fail | Step 3 still executes; incident created; Discord output notes annotation failure |
| IT-07 | Step 3b: all annotation updates fail | Runbook completes; Discord output notes update failure; incident still valid |

### Severity override

| IT-08 | Discord trigger includes "severity:critical" | Incident created with severity `critical` |
| IT-09 | No severity in trigger | Incident created with default severity `high` |

---

## Acceptance Criteria Verification

| AC | Check |
|---|---|
| AC-1 | Manual: type trigger in Discord, verify incident link returned |
| AC-2 | Unit: `stepQueryLogs` SQ-01 — CPL query includes `service:<service> @since:<time>` |
| AC-3 | Unit: `stepAnnotateLogs` SA-01 — 5 entries annotated with `annotation_type: incident` and `author_type: ai_agent` (injected by Pantheon executor) |
| AC-4 | Unit: `stepCreateIncident` SC-05 — `alert_source = "citadel"` on created incident |
| AC-5 | Integration: IT-04 — empty log result → halt, no incident created |
| AC-6 | Integration: IT-06 — annotation failures → incident still created |
| AC-7 | Integration: IT-01 — `{PENDING}` replaced with real incident ID in Step 3b |
| AC-8 | Unit: SC-03 — severity override respected |
| AC-9 | Integration: IT-02 — runbook in `GET /api/v1/runbooks` |
| AC-10 | `go test ./...` passes (T14 task) |

---

## Out of Scope for QA

- End-to-end Discord integration (requires live Discord bot environment — covered by milestone UAT)
- Live Citadel API calls (mocked in all test cases)
- Frontend UI (no UI in this feature)
- Performance / load testing
