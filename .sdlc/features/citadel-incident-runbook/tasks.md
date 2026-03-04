# Tasks: Create Incident from Citadel Logs Runbook

## T1 — Define `CitadelIncidentRunbookCtx` struct

Create `internal/runbooks/citadel_incident.go`. Define the `CitadelIncidentRunbookCtx` struct with fields for all inputs (service, time, severity) and accumulated step outputs (logs, executed query, log count, annotation IDs, incident ID, incident URL). Add the `CitadelLogEntry` local type if not already imported from the tool executor output types.

## T2 — Implement `selectKeyEntries` helper

In `citadel_incident.go`, implement `selectKeyEntries(logs []CitadelLogEntry, maxN int) []CitadelLogEntry`. Priority order: error/fatal first, then warn, then fill with first+last entries from remaining. Cap at maxN. Write unit tests in `citadel_incident_test.go` covering: empty input, fewer entries than maxN, entries exceeding maxN, mixed levels.

## T3 — Implement `stepQueryLogs`

Implement the Step 1 function. Constructs CPL query as `service:<service> @since:<time>`, calls `ToolExecutor.Run("citadel_query_logs", ...)` with limit 50. Populates `ctx.Logs`, `ctx.ExecutedQuery`, `ctx.LogCount`. Returns error (halts runbook) if executor fails or returns empty logs. Unit test with mock executor: success, API error, empty result.

## T4 — Implement `stepAnnotateLogs`

Implement the Step 2 function. Calls `selectKeyEntries(ctx.Logs, 5)`. For each selected entry, calls `ToolExecutor.Run("citadel_annotate_log", ...)` with `annotation_type: "incident"` and content `"linked to incident-{PENDING} — triage for <service> around <time>"`. Collects successful `annotation_id` values into `ctx.AnnotationIDs`. Logs warnings on per-entry failures but always returns nil (non-fatal). Unit test: partial failure, total failure, all success.

## T5 — Implement `buildSummary` helper

Implement `buildSummary(ctx *CitadelIncidentRunbookCtx) string`. Produces the structured Markdown summary with service, time window, executed query, log count, annotation IDs list, synthesized key findings (top 3–5 distinct error messages from logs), and episode link (first log entry's `episode_id`, or "none"). Write unit tests for: episode present, episode absent, no annotations.

## T6 — Implement `synthesizeFindings` helper

Implement `synthesizeFindings(logs []CitadelLogEntry) string`. Extracts up to 5 distinct error messages from the log entries (deduplicated by message text). Returns a Markdown bullet list. Unit test: deduplication, fewer than 5 unique messages, empty input.

## T7 — Implement `stepCreateIncident`

Implement the Step 3 function. Calls `buildSummary(ctx)`, then `IncidentService.Create(...)` with title `[Citadel] <service> anomaly around <time>`, severity from `ctx.Severity`, status `investigating`, alert_source `"citadel"`. Populates `ctx.IncidentID` and `ctx.IncidentURL`. Returns error on failure (halts runbook). Unit test with mock IncidentService: success, DB error.

## T8 — Implement `stepUpdateAnnotations`

Implement the Step 3b function. For each entry in `selectKeyEntries(ctx.Logs, 5)` that has a corresponding annotation ID (by index), calls `ToolExecutor.Run("citadel_annotate_log", ...)` with updated content replacing `{PENDING}` with the real `incident-<ctx.IncidentID>`. Logs warnings on failures, always returns nil. Unit test: full replacement, partial failure.

## T9 — Assemble `CitadelCreateIncidentRunbook` definition

In `citadel_incident.go`, declare `var CitadelCreateIncidentRunbook = RunbookDefinition{...}` with all four steps wired to their functions using correct `OnFailure` modes (`HaltOnFailure` for Steps 1 and 3, `WarnContinue` for Steps 2 and 3b). Include `TriggerPattern`, `DisplayName`, `RequiresApproval: false`, and the parameter JSON schema.

## T10 — Register runbook in `registry.go`

Add `Register(CitadelCreateIncidentRunbook)` to `internal/runbooks/registry.go`'s `init()` function (or equivalent registration mechanism). Verify `GET /api/v1/runbooks` includes the new runbook name and description.

## T11 — Discord output formatting

Add a `formatDiscordResponse(ctx *CitadelIncidentRunbookCtx, warnings []string) string` helper. Produces the success message with incident URL, severity, log count, annotated count, and episode. Produces the "no logs found" message on halt. Includes partial warning line when annotation count is less than expected. Wire into the runbook's Discord reply path (via existing runbook executor hooks).

## T12 — Integration test: full runbook execution

In `citadel_incident_test.go`, write an integration-level test that wires a mock Citadel ToolExecutor (returns canned logs and annotation IDs) and mock IncidentService (returns a fake incident). Execute all four steps in sequence and assert: incident is created with correct fields, annotation IDs match, summary contains expected log count, Discord output contains the incident URL.

## T13 — Integration test: edge cases

Write tests for: (a) empty log result → halt before Step 2, no incident created; (b) all Step 2 annotations fail → incident still created, warning in Discord output; (c) Step 3 (create incident) fails → runbook halts, no Step 3b executed; (d) severity override from trigger → incident created with specified severity.

## T14 — Verify `go test ./...` passes

Run `go test ./internal/runbooks/...` and confirm all new tests pass. Run `go vet ./...` and `golint ./...` (or equivalent linter) with no new errors.
