# Spec: Create Incident from Citadel Logs Runbook

## Summary

Implement a Pantheon Runbook that automates the full triage loop from Citadel log evidence to a filed Pantheon incident. The runbook executes three sequential steps: query the relevant logs, annotate key entries linking them to the incident, and create the Pantheon incident with a log summary and Citadel episode link. It is triggered in Discord by a natural-language command and models the multi-step workflow as a Pantheon runbook rather than a monolithic tool call.

---

## Background

With `citadel-app-registration`, `citadel-query-logs-tool`, and `citadel-annotate-log-tool` in place, Pantheon agents can now read and annotate Citadel logs as individual tool calls. The missing layer is orchestration: connecting those tools into a coherent incident-creation workflow that a developer can trigger from Discord without manually running each step.

Pantheon's existing Runbook model (used for deployment gate checks, DB migration gates, etc.) supports sequential step execution with shared context passed between steps. This feature uses that model to wire the three Citadel operations into a single composable runbook.

---

## Runbook Definition

**Name:** `citadel_create_incident_from_logs`
**Display Name:** Create Incident from Citadel Logs
**Discord trigger pattern:** `create incident from logs around <time> for <service>`
**Requires approval:** `false` (creating a Pantheon incident is a low-risk write; destructive operations are out of scope)

### Step 1: Query Citadel Logs

**Command:** `citadel_query_logs`

**Input construction from trigger:**
- `query`: `service:<service> @since:<time>` — the `<service>` and `<time>` tokens are extracted from the Discord trigger by the Pantheon agent using the runbook's parameter schema.
- `time_range`: `1h` (default; the `<time>` anchor is used as `@since`, not `@last`)
- `limit`: `50`

**Output used by subsequent steps:**
- `logs[]` — full log entry array (passed as runbook context variable `$logs`)
- `metadata.executed_query` — echoed in the final incident summary

**Step failure behavior:** If `citadel_query_logs` returns `ok: false` or `logs` is empty, the runbook halts and reports the error to Discord. No annotation or incident creation is attempted.

---

### Step 2: Annotate Key Log Entries

**Command:** `citadel_annotate_log` (called once per key entry)

**Entry selection:** The Pantheon agent (not the runbook engine) selects the most relevant entries from `$logs` — typically error-level entries, entries with matching `trace_id`, or the first and last entries in the time window. Selection limit: up to 5 entries to avoid annotation spam.

**Annotation input:**
```json
{
  "log_id": "<entry.id from $logs>",
  "content": "linked to incident-{PENDING} — queried during triage for <service> around <time>",
  "annotation_type": "incident"
}
```

The `{PENDING}` placeholder in `content` is replaced in a post-creation pass (Step 3b) after the incident ID is known. This is consistent with how Pantheon's runbook engine handles forward references: it defers the update or documents the placeholder as acceptable.

**Output used by subsequent steps:**
- `annotation_id[]` — collected list of annotation IDs for inclusion in the incident summary

**Step failure behavior:** Annotation failures are non-fatal. If `citadel_annotate_log` fails for an individual entry, the runbook logs a warning and continues. If all annotations fail, the runbook logs a warning but proceeds to Step 3 (incident creation is not blocked by annotation failure).

---

### Step 3: Create Pantheon Incident

**Command:** Pantheon `CreateIncident` API (internal, not a Citadel tool call)

**Incident fields:**
- `title`: `[Citadel] <service> anomaly around <time>`
- `severity`: `high` (default; the Discord trigger may optionally override with `severity:<level>`)
- `status`: `investigating`
- `alert_source`: `"citadel"`
- `summary`: Structured summary block:

```
## Citadel Log Summary

**Service:** <service>
**Time window:** <time> (±1h)
**Query:** `<metadata.executed_query>`
**Logs reviewed:** <count> entries
**Annotations:** <annotation_id list>

### Key findings
<agent-synthesized 3–5 bullet points from $logs>

### Citadel Episode Link
<episode_id from first log entry, if present>
```

**Post-creation annotation update (Step 3b):**
After the incident is created and an `incident_id` is returned, the runbook performs a patch-annotation pass: for each annotation created in Step 2, it calls `citadel_annotate_log` again with the same `log_id` and updated `content` replacing `{PENDING}` with the real `incident-<id>`. This is a best-effort pass — failures are logged but do not affect the runbook result.

**Output to Discord:**
```
Incident created: [<title>](<pantheon incident URL>)
  Severity: high
  <count> log entries reviewed, <annotation_count> annotated in Citadel
  Citadel episode: <episode_id or "none">
```

---

## Runbook Parameter Schema

```json
{
  "type": "object",
  "required": ["service", "time"],
  "properties": {
    "service": {
      "type": "string",
      "description": "The service name to query logs for (e.g. 'auth', 'api', 'payments')"
    },
    "time": {
      "type": "string",
      "description": "Anchor time for the log query — ISO 8601 or natural language (e.g. '2026-03-03T10:00:00Z', '10 minutes ago'). Pantheon agent normalizes to ISO 8601 before passing to citadel_query_logs."
    },
    "severity": {
      "type": "string",
      "enum": ["low", "medium", "high", "critical"],
      "default": "high",
      "description": "Pantheon incident severity. Defaults to high."
    }
  }
}
```

---

## Discord Trigger

**Pattern:** `create incident from logs around <time> for <service>`

**Examples:**
- `create incident from logs around 10:30am for auth`
- `create incident from logs around 2026-03-03T10:00:00Z for payments`
- `create incident from logs around 5 minutes ago for api severity:critical`

Pantheon's agent extracts `service`, `time`, and optionally `severity` from the trigger. If extraction fails, the agent asks a clarifying question before executing the runbook.

---

## Runbook Registration

The runbook is registered in Pantheon via the existing `RunbookDefinition` mechanism:

```json
{
  "name": "citadel_create_incident_from_logs",
  "display_name": "Create Incident from Citadel Logs",
  "description": "Query Citadel logs for a time window, annotate key entries, and create a Pantheon incident with a log summary.",
  "trigger_pattern": "create incident from logs around {time} for {service}",
  "parameters": { /* schema above */ },
  "steps": [
    { "id": "query_logs",   "command": "citadel_query_logs",    "on_failure": "halt" },
    { "id": "annotate",     "command": "citadel_annotate_log",  "on_failure": "warn_continue" },
    { "id": "create_incident", "command": "CreateIncident",     "on_failure": "halt" },
    { "id": "update_annotations", "command": "citadel_annotate_log", "on_failure": "warn_continue" }
  ],
  "requires_approval": false
}
```

---

## Implementation Location

This work lives in the **Pantheon** Go codebase:

- `internal/runbooks/citadel_incident.go` — step definitions and context wiring
- `internal/api/handlers/runbooks.go` — registration endpoint (if not already present)
- Skill instructions update in Pantheon's Discord agent profile: add the trigger pattern and parameter extraction instructions

---

## Dependencies

- `citadel-app-registration` — `AppRegistration` and `ToolCredential` for Citadel must exist
- `citadel-query-logs-tool` — Step 1 tool must be registered and executable
- `citadel-annotate-log-tool` — Step 2 tool must be registered and executable

---

## Out of Scope (V1)

- Web UI for triggering the runbook (Discord only)
- Scheduling/recurring runbooks (on-demand only)
- Automatic runbook trigger from Citadel webhook (that is a separate automation layer on top of `citadel-webhook-handler`)
- Multi-service queries (one service per invocation)
- Rollback / undo runbook (incidents can be manually updated or closed)

---

## Acceptance Criteria

1. Saying `create incident from logs around <time> for <service>` in a Discord channel with Pantheon triggers the runbook and responds with a created incident link.
2. Step 1 queries Citadel with the correct CPL query incorporating the service and time anchor.
3. Step 2 annotates up to 5 key log entries with `annotation_type: "incident"` and `author_type: "ai_agent"`.
4. Step 3 creates a Pantheon incident with `alert_source = "citadel"` and a structured summary including the Citadel episode link (if available).
5. If `citadel_query_logs` returns no logs, the runbook halts and reports `"No logs found for <service> around <time>"` to Discord without creating an incident.
6. Annotation failures (Steps 2 and 3b) do not halt runbook execution — the incident is still created.
7. The post-creation annotation update (Step 3b) replaces `{PENDING}` with the real incident ID for successfully annotated entries.
8. If `severity` is specified in the Discord trigger, the created incident uses that severity level.
9. The runbook is listed in `GET /api/v1/runbooks` with the correct name and description.
10. All new Go code passes `go test ./...`.
