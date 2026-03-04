# QA Plan: citadel_annotate_log Agent Tool

## Scope

Verify the `citadel_annotate_log` Pantheon App Platform tool:
1. Parameter validation rejects invalid inputs before any HTTP call
2. Config template always hardcodes `author_type: "ai_agent"`
3. Response mapping correctly transforms Citadel HTTP responses into agent-readable results
4. Credential reuse — no duplicate `citadel` credential created
5. `requires_approval = true` enforced by executor
6. Skill instructions cover all five annotation types
7. Integration with the executor pipeline produces correct audit records

---

## QA-1: Parameter Validation — Valid Inputs

**Test:** Call `ValidateParams` with all required fields present and a valid `annotation_type`.

**Inputs:**
```json
{ "log_id": "log_abc", "content": "Root cause: null pointer in auth handler", "annotation_type": "root_cause" }
```

**Expected:** No error returned.

**Run:** `go test ./internal/appplatform/tools/... -run TestValidateParams_Valid`

---

## QA-2: Parameter Validation — Missing log_id

**Test:** Call `ValidateParams` with `log_id` omitted.

**Inputs:** `{ "content": "note", "annotation_type": "note" }`

**Expected:** Error containing "log_id is required".

**Run:** `go test ./internal/appplatform/tools/... -run TestValidateParams_MissingLogId`

---

## QA-3: Parameter Validation — Missing content

**Test:** Call `ValidateParams` with `content` empty string.

**Expected:** Error containing "content is required".

---

## QA-4: Parameter Validation — Content Exceeds 4096 Characters

**Test:** Call `ValidateParams` with `content` set to a 4097-character string.

**Expected:** Error containing "4096 character limit".

---

## QA-5: Parameter Validation — Invalid annotation_type

**Test:** Call `ValidateParams` with `annotation_type: "severity"`.

**Expected:** Error listing valid values: `note, bug, root_cause, false_positive, incident`.

---

## QA-6: All Five Enum Values Pass Validation

**Test:** For each of `note`, `bug`, `root_cause`, `false_positive`, `incident`: call `ValidateParams` with that annotation_type.

**Expected:** All five pass without error.

**Run:** `go test ./internal/appplatform/tools/... -run TestValidateParams_AllFiveEnumValues`

---

## QA-7: author_type Hardcoded in Template

**Test:** Render the config template with all valid params, including an attempt to pass `author_type: "human"` as an extra param.

**Expected:** The rendered POST body contains `"author_type": "ai_agent"` — not `"human"`. The template must not expose `{{.author_type}}` as a substitution point.

**Run:** `go test ./internal/appplatform/tools/... -run TestMapResponse_AuthorTypeHardcoded`

---

## QA-8: Response Mapping — 201 Success

**Test:** Provide mock HTTP 201 response:
```json
{
  "id": "ann_abc123xyz",
  "log_id": "log_000000000000abcd",
  "annotation_type": "root_cause",
  "created_at": "2026-03-03T10:15:00Z",
  "author_type": "ai_agent"
}
```

**Expected:** `MapResponse` returns a ToolResult with:
- `annotation_id: "ann_abc123xyz"`
- `log_id: "log_000000000000abcd"`
- `created_at: "2026-03-03T10:15:00Z"`
- `author_type: "ai_agent"`
- `citadel_url` containing both the log_id and annotation_id

---

## QA-9: Response Mapping — 404 Not Found

**Test:** Provide mock HTTP 404 from Citadel.

**Expected:** `MapResponse` returns a tool error with message: "Log entry not found in Citadel. Verify the log_id from citadel_query_logs."

---

## QA-10: Response Mapping — 422 Unprocessable

**Test:** Provide mock HTTP 422 from Citadel.

**Expected:** Tool error: "Invalid annotation_type. Must be one of: note, bug, root_cause, false_positive, incident."

---

## QA-11: Response Mapping — 401 Unauthorized

**Test:** Provide mock HTTP 401 from Citadel.

**Expected:** Tool error: "Citadel API key rejected. Check the citadel ToolCredential."

---

## QA-12: Credential Reuse — No Duplicate citadel Credential

**Test:** Inspect the tool registry at startup. Verify there is exactly one `ToolCredential` with `service_name: "citadel"`.

**Expected:** Both `citadel_query_logs` and `citadel_annotate_log` reference the same credential record. The registry contains one citadel credential, not two.

**Check:** `grep -r "citadel" internal/config/tools.go | grep -i credential` — should show one definition, two references.

---

## QA-13: requires_approval Enforced

**Test:** Invoke `citadel_annotate_log` through the executor without an approval gate in place.

**Expected:** Executor returns an "approval required" response before dispatching the HTTP call. The Citadel POST is never sent without approval.

---

## QA-14: AppAction Audit Record Written

**Test (integration):** Call the full executor pipeline with a mock Citadel server returning 201. After the call, query the `AppAction` audit table.

**Expected:** One `AppAction` record exists with:
- `tool_name: "citadel_annotate_log"`
- `status: "success"`
- `params` field present but with `content` redacted (content may be large/sensitive)

---

## QA-15: Skill Instructions Cover All Five annotation_type Values

**Test:** Read `agent_skills/citadel_pantheon_discord.md`.

**Expected:** The `citadel_annotate_log` section mentions all five annotation types: `note`, `bug`, `root_cause`, `false_positive`, `incident` — each with usage guidance.

**Check:** `grep -c "root_cause\|false_positive\|incident\|bug\|note" agent_skills/citadel_pantheon_discord.md` — count ≥ 5 distinct entries in the citadel_annotate_log section.

---

## QA-16: End-to-End Smoke Test — Agent Annotation Flow

**Scenario:** Simulate the full agent → executor → Citadel → response chain.

1. Agent calls `citadel_query_logs` → receives log entries with `id` fields.
2. Agent selects `log_id: "log_abc"` and calls `citadel_annotate_log` with `annotation_type: "root_cause"`.
3. Executor validates params, loads citadel credential, renders template, fires POST (mocked).
4. Mock returns 201 with annotation object.
5. Executor maps response and returns result to agent.

**Expected:** Agent receives `annotation_id` and `citadel_url`. The `citadel_url` links to the correct log entry and annotation anchor. No credential duplication. AppAction record written.

---

## Pass/Fail Criteria

All 16 QA checks must pass. The feature is ready for merge when:
- `go test ./internal/appplatform/tools/...` passes with 0 failures
- Integration smoke test (QA-16) passes with mocked Citadel
- Skill instructions verified (QA-15) — manual check on the skill file content
- No second `citadel` credential created (QA-12) — verified via config inspection
