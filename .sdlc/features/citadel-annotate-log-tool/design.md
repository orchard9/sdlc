# Design: citadel_annotate_log Agent Tool

## Overview

This is a Pantheon App Platform tool addition — backend-only with no UI. The design follows the identical pattern established by `citadel_query_logs`, which already introduced the `citadel` ToolCredential and ToolExecutor pipeline. No new infrastructure is required.

## File Layout

```
internal/appplatform/
  tools/
    citadel_annotate_log.go      ← NEW: ToolDefinition registration + parameter validator
    citadel_query_logs.go        ← existing (reference pattern)

internal/config/
  tools.go                       ← existing, registers all tool definitions at startup

internal/api/handlers/
  appplatform.go                 ← existing, executor calls — no changes needed

agent_skills/
  citadel_pantheon_discord.md    ← existing skill file, append citadel_annotate_log section
```

## ToolDefinition Data Shape

The `ToolDefinition` struct (already in `internal/appplatform/types.go`) covers everything needed. For `citadel_annotate_log`:

```
Name:             "citadel_annotate_log"
Description:      "Attach a structured annotation to a Citadel log entry."
CredentialRef:    "citadel"           ← reuses existing citadel api_key credential
RiskLevel:        medium
RequiresApproval: true
Parameters:       [log_id, content, annotation_type]
ConfigTemplate:   (see below)
```

### Parameter Definitions

```
log_id:
  type:        string
  required:    true
  description: "Citadel log entry ID to annotate (from citadel_query_logs results)"

content:
  type:        string
  required:    true
  max_length:  4096
  description: "Annotation body — plain text or Markdown"

annotation_type:
  type:        string
  required:    true
  enum:        [note, bug, root_cause, false_positive, incident]
  description: "Classification of this annotation"
```

### Config Template

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

`author_type` is literal in the template — the executor will not substitute it from agent parameters. This prevents agents from impersonating other author types.

## Validation Logic

Client-side validation runs before the executor fires the HTTP call:

```
validate(params):
  if log_id is empty:
    return error "log_id is required"
  if content is empty:
    return error "content is required"
  if len(content) > 4096:
    return error "content exceeds 4096 character limit"
  if annotation_type not in [note, bug, root_cause, false_positive, incident]:
    return error "invalid annotation_type: must be one of note, bug, root_cause, false_positive, incident"
```

This is implemented as a `ValidateParams(params map[string]any) error` method on the tool's registration struct, called by the executor before template rendering.

## Response Mapping

The executor receives the raw HTTP response from Citadel. The tool's `MapResponse` function transforms it:

```
Citadel 201 Created → success result:
  {
    annotation_id: response.id,
    log_id:        response.log_id,
    annotation_type: response.annotation_type,
    created_at:    response.created_at,
    author_type:   "ai_agent",
    citadel_url:   "https://citadel.orchard9.ai/logs/{log_id}#ann_{annotation_id}"
  }

Citadel 404 Not Found → tool error:
  "Log entry not found in Citadel. Verify the log_id from citadel_query_logs."

Citadel 422 Unprocessable → tool error:
  "Invalid annotation_type. Must be one of: note, bug, root_cause, false_positive, incident."

Citadel 401 Unauthorized → tool error:
  "Citadel API key rejected. Check the citadel ToolCredential."

Other 4xx/5xx → tool error:
  "Citadel API error {status}: {body}"
```

## Credential Sharing Sequence

```
Agent → Executor.Execute("citadel_annotate_log", {log_id, content, annotation_type})
  └─ Executor.LoadCredential("citadel")          ← same credential as citadel_query_logs
       └─ CredentialService.Decrypt(citadel.api_key)
  └─ Executor.ValidateParams(params)             ← annotation_type enum check
  └─ Executor.RenderTemplate(config_template, {credential, tenant_id, ...params})
  └─ Executor.Dispatch(POST /api/v1/annotations, rendered_body)
  └─ Executor.MapResponse(201 → success, 4xx → error)
  └─ Executor.RecordAppAction(tool_name, params_redacted, result)
```

No new code paths in the executor. This uses the existing pipeline end-to-end.

## Skill Instructions Addition

Append to `agent_skills/citadel_pantheon_discord.md` after the `citadel_query_logs` section:

```markdown
### citadel_annotate_log

Attach a structured annotation to a Citadel log entry to persist agent insights.

**When to use:**
- Root cause identified: persist the analysis linked to the specific log entry.
- False positive confirmed: mark the entry so future agents don't re-investigate it.
- Linking to Pantheon incident: use annotation_type "incident" and include the incident ID in content.
- Bug confirmed: use annotation_type "bug" and name the affected component.

**Parameters:**
- `log_id` — from citadel_query_logs results (each log entry has an "id" field)
- `content` — your annotation text (Markdown supported, max 4096 chars)
- `annotation_type` — one of: note | bug | root_cause | false_positive | incident

**annotation_type guidance:**
- `root_cause` — include the code path or condition causing the error
- `false_positive` — state why the event is not actionable
- `incident` — include the Pantheon incident ID: "Linked to INC-1234: auth failure cascade"
- `bug` — confirm real bug + affected component: "Bug in auth service token refresh"
- `note` — general observation that doesn't fit other types

**Important:** Always get log_id from citadel_query_logs first. Never guess a log_id.
```

## No Database Changes

No new tables, migrations, or schema changes. The `ToolDefinition` and `ToolCredential` models are fully in-memory/config at startup. The existing `AppAction` audit table already records all tool invocations.

## Test Coverage

```
Unit tests (tools/citadel_annotate_log_test.go):
  TestValidateParams_Valid           — all required params present, valid annotation_type
  TestValidateParams_MissingLogId    — empty log_id → error
  TestValidateParams_MissingContent  — empty content → error
  TestValidateParams_ContentTooLong  — 4097 chars → error
  TestValidateParams_InvalidType     — "severity" → error
  TestValidateParams_AllEnumValues   — each of 5 valid types passes

Integration test (via existing executor mock):
  TestExecute_Success                — mock 201, verify response mapping
  TestExecute_NotFound               — mock 404, verify error message
  TestExecute_UnauthorizedCredential — mock 401, verify error message
  TestExecute_AuthorTypeHardcoded    — verify author_type is "ai_agent" in rendered body
```

## Acceptance Criteria Mapping

| Criterion | Design Element |
|---|---|
| Agent invokes tool, gets annotation_id + created_at | Response mapping from 201 → success result |
| Reuses citadel credential, no duplicate | CredentialRef = "citadel", shared with query_logs |
| author_type always "ai_agent" | Hardcoded literal in config template |
| requires_approval = true | ToolDefinition field |
| annotation_type validated before HTTP | ValidateParams enum check |
| 404 → clear error message | Response mapping for 404 |
| Skill includes all 5 annotation_type values | Skill instructions addition |
| RiskLevel = medium | ToolDefinition field |
