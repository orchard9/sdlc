# Security Audit: Create Incident from Citadel Logs Runbook

## Audit Scope

Security review of the `citadel_create_incident_from_logs` Pantheon runbook implementation. Primary surfaces: Discord trigger parameter parsing, Citadel API calls via ToolExecutor, Pantheon incident creation, and credential handling.

---

## Finding 1 — Discord Trigger Parameter Injection

**Severity:** Medium
**Surface:** `service` and `time` parameters extracted from Discord message text and inserted directly into a CPL query string (`service:<service> @since:<time>`)

**Risk:** An attacker with Discord access could inject CPL metacharacters into the `service` or `time` parameters to manipulate the Citadel query (e.g., `service:auth OR service:*` to broaden the query scope). This could expose log data from unintended services.

**Finding:** The `service` field is passed as-is into the CPL query template without sanitization.

**Resolution:** Add input validation before constructing the CPL query:
- `service`: allow only `[a-zA-Z0-9_\-]+` (alphanumeric, underscore, hyphen). Reject and report any other characters.
- `time`: validate as ISO 8601 timestamp or restricted natural-language pattern (e.g., `\d+ (minutes?|hours?) ago`). Reject free-form strings.
- Return a clear Discord error message on validation failure: `"Invalid service name. Use alphanumeric characters only."`

**Action:** Fix in-cycle. Add `validateRunbookParams(params)` function in `citadel_incident.go` called before Step 1. Add unit tests for injection attempts.

---

## Finding 2 — Annotation Content: User-Controlled String in Citadel

**Severity:** Low
**Surface:** `content` field of `citadel_annotate_log` includes `ctx.Service` and `ctx.Time` (user-controlled via Discord trigger)

**Risk:** The annotation content is stored in Citadel and may be rendered in the Citadel UI. If Citadel renders annotation content as HTML, this could be a stored XSS vector. However, since annotations are only visible to authenticated Citadel users (not public), the blast radius is limited.

**Resolution:** After applying F1's service name sanitization, the `service` field is already constrained to alphanumeric. The `time` field after validation is ISO 8601 or a restricted pattern — no XSS risk from these fields. No further action needed if F1's validation is implemented.

**Action:** Accepted — contingent on F1 resolution. Document that annotation content inherits the service/time input validation boundary.

---

## Finding 3 — No Authorization Check on Discord Trigger

**Severity:** Medium
**Surface:** The Discord trigger `create incident from logs around <time> for <service>` is matched against all messages in configured channels

**Risk:** Any Discord user in the channel can trigger incident creation, including external contributors or accidentally by mistake. This could create noise incidents or, if an attacker has channel access, spam the incident queue.

**Resolution:** Pantheon's existing Discord bot framework supports role-based command authorization (checked against Discord role IDs). The `citadel_create_incident_from_logs` runbook must require the `incident-responder` role (or equivalent configured role) before execution.

**Action:** Fix in-cycle. Register the runbook with `required_discord_roles: ["incident-responder"]` in the `RunbookDefinition`. Add test: trigger from non-authorized user → bot rejects with permission error.

---

## Finding 4 — Credential Handling (delegated to ToolExecutor)

**Severity:** Low (no new credential surface)
**Surface:** Citadel API key used in Steps 1, 2, and 3b

**Risk:** The runbook does not handle credentials directly — it delegates to Pantheon's `ToolExecutor`, which decrypts from the `ToolCredential` store. No new credential handling code is introduced in this feature.

**Resolution:** No action needed. The existing ToolExecutor credential path (from `citadel-app-registration`) is the correct pattern and has its own security properties.

**Action:** Accepted. No code changes needed for this finding.

---

## Finding 5 — Incident Creation is a Write Operation with No Approval Gate

**Severity:** Low
**Surface:** `stepCreateIncident` creates a Pantheon incident without any approval gate

**Risk:** The spec explicitly sets `requires_approval: false`. Creating incidents is a low-stakes write (incidents can be closed/edited). An attacker would need Discord channel access + `incident-responder` role (after F3 fix) to trigger creation. Risk is low.

**Resolution:** Accepted. The decision to omit an approval gate is correct for this workflow — the human invoking the Discord trigger is the approval signal. F3's role requirement provides the authorization control.

**Action:** Accepted. No code changes.

---

## Finding 6 — `synthesizeFindings` Could Leak Sensitive Log Data into Incident Summary

**Severity:** Low
**Surface:** `synthesizeFindings` extracts up to 5 distinct error messages from log entries and embeds them in the Pantheon incident summary

**Risk:** Citadel log messages may contain PII, tokens, or secrets (e.g., `"Auth failed for user: user@company.com token: sk_..."`) that get copied verbatim into the Pantheon incident record, which may have broader visibility than Citadel.

**Resolution:** This is an inherent property of connecting observability and incident management systems. The risk is accepted under the assumption that Pantheon incidents are access-controlled at the org level. Add a note in the runbook's documentation: "Log messages included in the incident summary inherit Citadel's log data — ensure Citadel logs are appropriately redacted for sensitive data before using this runbook."

**Action:** Accepted with documentation note. No code change needed.

---

## Summary

| Finding | Severity | Action |
|---|---|---|
| F1 — CPL parameter injection via service/time | Medium | Fix in-cycle: add `validateRunbookParams` |
| F2 — User-controlled annotation content | Low | Accepted (contingent on F1) |
| F3 — No Discord role authorization | Medium | Fix in-cycle: add `required_discord_roles` |
| F4 — Credential handling delegated | Low | Accepted |
| F5 — No approval gate on incident creation | Low | Accepted |
| F6 — Log data in incident summary | Low | Accepted with documentation note |

**Verdict:** Two medium findings (F1, F3) require in-cycle fixes before the audit can be approved. Four low findings accepted as-is with documentation.
