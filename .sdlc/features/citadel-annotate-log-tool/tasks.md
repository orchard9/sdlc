# Tasks: citadel_annotate_log Agent Tool

## T1 — Create ToolDefinition for citadel_annotate_log

**File:** `internal/appplatform/tools/citadel_annotate_log.go`

Create the tool registration struct implementing the `Tool` interface:
- Name: `citadel_annotate_log`
- Description: "Attach a structured annotation to a Citadel log entry."
- CredentialRef: `"citadel"` (reuses existing credential from citadel_query_logs)
- RiskLevel: `medium`
- RequiresApproval: `true`
- Parameters: `log_id` (string, required), `content` (string, required, max 4096), `annotation_type` (string, required, enum)
- ConfigTemplate: POST to `https://citadel.orchard9.ai/api/v1/annotations` with `author_type` hardcoded to `"ai_agent"`

## T2 — Implement ValidateParams for annotation_type enum and content length

In `citadel_annotate_log.go`, implement `ValidateParams(params map[string]any) error`:
- Reject empty `log_id`
- Reject empty `content`
- Reject `content` longer than 4096 characters
- Reject `annotation_type` not in `[note, bug, root_cause, false_positive, incident]`
- Return descriptive error messages for each case

## T3 — Implement MapResponse for 201 success and 4xx error cases

In `citadel_annotate_log.go`, implement `MapResponse(statusCode int, body []byte) (ToolResult, error)`:
- `201` → success result with `annotation_id`, `log_id`, `annotation_type`, `created_at`, `author_type: "ai_agent"`, and computed `citadel_url`
- `404` → tool error: "Log entry not found in Citadel. Verify the log_id from citadel_query_logs."
- `422` → tool error: "Invalid annotation_type. Must be one of: note, bug, root_cause, false_positive, incident."
- `401` → tool error: "Citadel API key rejected. Check the citadel ToolCredential."
- Other → tool error: "Citadel API error {status}: {body}"

## T4 — Register citadel_annotate_log in tools.go at startup

**File:** `internal/config/tools.go`

Add `citadel_annotate_log` to the tool registry alongside `citadel_query_logs`. Ensure the tool is available to the executor at startup without requiring config file changes.

## T5 — Write unit tests for ValidateParams

**File:** `internal/appplatform/tools/citadel_annotate_log_test.go`

Tests:
- `TestValidateParams_Valid` — all params present and valid
- `TestValidateParams_MissingLogId` — empty `log_id` → error
- `TestValidateParams_MissingContent` — empty `content` → error
- `TestValidateParams_ContentTooLong` — 4097-char content → error
- `TestValidateParams_InvalidAnnotationType` — `"severity"` → error
- `TestValidateParams_AllFiveEnumValues` — each of `note`, `bug`, `root_cause`, `false_positive`, `incident` passes

## T6 — Write unit tests for MapResponse

In the same test file:
- `TestMapResponse_201Success` — mock 201 body, verify all fields in returned ToolResult
- `TestMapResponse_404` — verify user-facing error message
- `TestMapResponse_422` — verify user-facing error message
- `TestMapResponse_401` — verify credential error message
- `TestMapResponse_AuthorTypeHardcoded` — verify the rendered config template always contains `"author_type": "ai_agent"` regardless of what params are passed

## T7 — Update Pantheon Discord agent skill instructions

**File:** `agent_skills/citadel_pantheon_discord.md`

Append the `citadel_annotate_log` section after the `citadel_query_logs` section:
- When to use (root cause, false positive, incident linking, bug confirmation)
- Parameter descriptions
- annotation_type guidance for all five values
- Note: always get `log_id` from `citadel_query_logs` results first

## T8 — Verify integration: executor pipeline smoke test

In the existing executor integration test file (or a new `citadel_annotate_log_integration_test.go`):
- Wire `citadel_annotate_log` through the full executor pipeline with a mock HTTP server
- Verify `author_type: "ai_agent"` appears in the actual POST body sent (not just template)
- Verify `AppAction` audit record is written with tool name and redacted params
- Verify `requires_approval = true` causes the executor to gate on approval before firing
