# Spec: citadel_annotate_log Agent Tool

## Problem

Pantheon agents operating in Discord can query Citadel logs (via `citadel_query_logs`) and receive webhook-triggered incident alerts (via the Citadel webhook handler), but there is no way for agents to write back to Citadel. When an agent identifies a root cause, classifies a false positive, or links a log entry to a Pantheon incident, that insight exists only in the conversation — it is never persisted in Citadel's log record. The observability and task layers remain one-directional.

## Solution

Register `citadel_annotate_log` as a Pantheon App Platform tool. This enables agents to attach structured annotations to Citadel log entries, creating a durable record linking agent-derived insights (root cause, false positive classification, incident notes) to the specific log events that triggered them. Citadel's annotation model explicitly supports `author_type: "ai_agent"` — this feature uses that extension point.

## Scope

This feature delivers one tool:

**`citadel_annotate_log`** — attach a structured annotation to a Citadel log entry.

### Parameters

| Parameter         | Type   | Required | Description |
|---|---|---|---|
| `log_id`          | string | Yes      | Citadel log entry ID to annotate |
| `content`         | string | Yes      | Annotation body (max 4096 chars). Plain text or Markdown. |
| `annotation_type` | string | Yes      | Classification: `note` \| `bug` \| `root_cause` \| `false_positive` \| `incident` |

### Return Value

```json
{
  "annotation_id": "ann_abc123xyz",
  "log_id": "log_000000000000abcd",
  "annotation_type": "root_cause",
  "created_at": "2026-03-03T10:15:00Z",
  "author_type": "ai_agent",
  "citadel_url": "https://citadel.orchard9.ai/logs/log_000000000000abcd#ann_abc123xyz"
}
```

### annotation_type Semantics

| Value           | Meaning |
|---|---|
| `note`          | General observation, not a classification |
| `bug`           | Confirms the log event reflects a real bug requiring a fix |
| `root_cause`    | Agent has identified the root cause; content includes the analysis |
| `false_positive`| Log event is not an actionable error (noise, expected behavior) |
| `incident`      | Log event is linked to an active Pantheon incident; content includes incident ID |

## Architecture

The tool is implemented inside Pantheon using the existing App Platform, matching the pattern established by `citadel_query_logs`.

### 1. ToolCredential

Reuses the `citadel` credential entry (type `api_key`) already created for `citadel_query_logs`. No new credential required — both tools share the same Citadel API key.

### 2. ToolDefinition

Describes the HTTP call to Citadel's `POST /api/v1/annotations`. Config template:

```json
{
  "url": "https://citadel.orchard9.ai/api/v1/annotations",
  "method": "POST",
  "headers": {
    "Authorization": "Bearer {{.credential}}",
    "Content-Type": "application/json",
    "X-Tenant-ID": "{{.tenant_id}}"
  },
  "body": {
    "log_id": "{{.log_id}}",
    "content": "{{.content}}",
    "annotation_type": "{{.annotation_type}}",
    "author_type": "ai_agent"
  }
}
```

`author_type` is hardcoded to `"ai_agent"` in the config template — agents cannot override it. This preserves Citadel's audit trail integrity.

### 3. ToolExecutor

Uses the existing `ToolExecutor` in `internal/appplatform/executor.go` without modification. The executor already handles credential injection, template rendering, HTTP dispatch, and `AppAction` audit records.

### 4. RiskLevel and Approval

- `requires_approval`: `true` — writing to Citadel is a mutating, persistent operation. The agent must confirm the annotation intent before submission.
- `RiskLevel`: `medium` — annotations are visible to the entire team in Citadel and cannot be deleted via API once created.

### 5. Skill Instructions

The Pantheon agent profile for Discord agents must include:

```
citadel_annotate_log — write a structured annotation to a Citadel log entry.
Use when:
  - You have identified the root cause of an error and want to persist it in Citadel.
  - You have confirmed an alert is a false positive.
  - You want to link a log entry to a Pantheon incident (use annotation_type: "incident", include incident ID in content).

annotation_type guidance:
  root_cause     — include your analysis; be specific about the code path or condition that caused the error
  false_positive — state why the event is not actionable (e.g., "healthcheck probe misclassified as error")
  incident       — include the Pantheon incident ID and a one-line description of the link
  bug            — confirm the log reflects a real bug; include affected component
  note           — for observations that don't fit other categories

Never annotate without the log_id. Get the log_id from citadel_query_logs results (each log entry includes an "id" field).
```

## Citadel API Contract

`POST /api/v1/annotations`

**Request body:**
```json
{
  "log_id": "string",
  "content": "string",
  "annotation_type": "note|bug|root_cause|false_positive|incident",
  "author_type": "ai_agent"
}
```

**Success response (201 Created):**
```json
{
  "id": "ann_abc123xyz",
  "log_id": "log_000000000000abcd",
  "annotation_type": "root_cause",
  "created_at": "2026-03-03T10:15:00Z",
  "author_type": "ai_agent"
}
```

**Error responses:**
- `404 Not Found` — `log_id` does not exist
- `422 Unprocessable Entity` — invalid `annotation_type`
- `401 Unauthorized` — bad or missing API key

The tool executor maps `201 Created` to a success result. `4xx` responses surface as tool errors to the agent.

## What This Does NOT Cover

- Editing or deleting existing annotations — Citadel's API does not expose mutation endpoints for annotations post-creation.
- Bulk annotation across multiple log IDs — use one `citadel_annotate_log` call per log entry.
- Annotation of Citadel error group (fingerprint) records — this annotates individual log entries only; fingerprint-level annotation is a future feature.
- UI for viewing agent annotations in Pantheon — agents create annotations in Citadel; the Citadel dashboard displays them.

## Acceptance Criteria

1. A Pantheon agent in Discord can invoke `citadel_annotate_log` with `log_id`, `content`, and `annotation_type`, and receive back an `annotation_id` and `created_at`.
2. The Citadel API key is loaded from the existing `citadel` ToolCredential (not duplicated).
3. `author_type` is always `"ai_agent"` in the POST body — agents cannot override it.
4. `requires_approval` is `true`; the agent must confirm before the tool fires.
5. `annotation_type` is validated client-side in the ToolDefinition before the HTTP call; invalid values return a tool error without hitting Citadel.
6. A `404` from Citadel (log_id not found) surfaces as a clear tool error: "Log entry not found in Citadel".
7. The Pantheon agent skill includes usage guidance for all five `annotation_type` values.
8. RiskLevel is `medium`.
9. The tool reuses the `citadel` ToolCredential without creating a second credential record.
