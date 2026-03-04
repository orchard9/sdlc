# Review: Register Citadel as a Pantheon App

## Summary

This feature establishes Pantheon's App Platform foundation: three new Postgres tables (`app_registrations`, `tool_definitions`, `tool_credentials`), six REST routes for CRUD management, an AES-256-GCM credential encryption service, and an extended tool executor that injects credentials and dispatches HTTP calls to Citadel.

The implementation is in the **Pantheon Go codebase** and does not modify sdlc. This review evaluates the design artifacts for correctness, completeness, and readiness for implementation.

---

## Findings

### Security

**[PASS] Credential encryption design is sound.**
HKDF-SHA256 key derivation with per-org salting + AES-256-GCM with random nonce is a well-established pattern. Each org's credential is isolated: compromise of one org's encrypted blob does not help with another.

**[PASS] Plaintext key never returned after storage.**
The `StoreCredential` handler response is explicitly defined to return only `id`, `app_id`, `org_slug`, `key_prefix`, and `created_at`. The `api_key` field is absent from the response type.

**[PASS] Citadel key format validation defined precisely.**
Regex `^ck_(prod|staging|dev)_[a-z0-9]{1,16}_[0-9a-f]{32}$` is specific and testable. All edge cases are covered in the QA plan.

**[PASS] Startup check for `PANTHEON_CREDENTIAL_KEY`.**
A `RequireCredentialKey()` check ensures the service won't silently start with uncryptable credentials.

**[NOTE] `author_type: "ai_agent"` injection is tool-name-specific.**
The design notes this as intentional for now and flags it for future generalization to an `inject_fields` map. This is acceptable for the initial implementation; a backlog item should be created to generalize when a second annotating app is added.

### Data Model

**[PASS] Three-table schema matches the spec.**
`app_registrations`, `tool_definitions`, `tool_credentials` with correct FKs, UNIQUE constraints, and CASCADE deletes. Indexes on `(app_id)` and `(app_id, org_slug)` are appropriate for lookup patterns.

**[PASS] `UNIQUE(app_id, org_slug)` on credentials enforces one active credential per org per app.**
This matches the design intent. Upsert or reject-on-duplicate behavior should be clearly defined in the handler (the QA plan flags this as "defined behavior" — the implementation must choose one).

**[MINOR] No `updated_at` on `tool_definitions` or `app_registrations`.**
Future metadata update operations would benefit from timestamps. Not a blocker for this feature since update routes are out of scope, but worth adding to the migration now to avoid a later migration.

### API Design

**[PASS] REST routes match the spec completely.**
All 6 routes are present with correct methods and path patterns. Error codes are enumerated in the design.

**[PASS] 409 conflict on duplicate app name is explicit.**
`UNIQUE(org_slug, name)` violation is mapped to a distinct error code `app_name_conflict`, not a generic 500.

**[PASS] Credential deletion is org-scoped.**
The `DELETE .../credentials/:cid` handler verifies org ownership before deletion, preventing cross-org deletion attacks.

### Testing

**[PASS] Unit test coverage is complete.**
11 credential unit tests + 9 executor unit tests cover all happy paths and error conditions.

**[PASS] Integration tests cover all CRUD operations and isolation.**
The QA plan includes isolation tests (Org A cannot read Org B's credential) and cascade deletion verification.

**[PASS] All 7 acceptance criteria have explicit test coverage.**
Each criterion in the spec maps to a row in the QA plan's verification table.

### Implementation Readiness

**[PASS] All artifacts are complete and internally consistent.**
Spec → Design → Tasks → QA Plan are aligned: no field names, error codes, or route paths differ between documents.

**[PASS] Dependencies are clear.**
This feature has no dependencies on other features in the milestone. `citadel-query-logs-tool` and `citadel-annotate-log-tool` depend on this feature's `ToolDefinition` records existing.

---

## Decisions

| Decision | Rationale |
|---|---|
| HKDF-SHA256 + AES-256-GCM for credential encryption | Industry standard; no external key management service needed for V1 |
| One active credential per org per app (UNIQUE constraint) | Simplifies executor lookup; rotation is handled by delete+create |
| `author_type: "ai_agent"` injected by executor, not caller | Citadel's author attribution is Pantheon's concern, not the agent's |
| No frontend UI in this feature | Credential management via API is sufficient for V1; UI is a follow-on |
| `PANTHEON_CREDENTIAL_KEY` as env var (not KMS) | Sufficient for current scale; KMS migration path is add-compatible |

---

## Action Items

1. **Add `updated_at` columns** to `app_registrations` and `tool_definitions` in the migration — negligible cost now, avoids a future migration.
2. **Define upsert vs. reject behavior** for duplicate `(app_id, org_slug)` credential — pick one and document it in the handler comment.
3. **Create backlog item** for `inject_fields` generalization in `AppRegistration` when a second annotating app is added.
4. **Document `PANTHEON_CREDENTIAL_KEY`** in Pantheon's `.env.example` and operational runbook before deploying to production.

Items 1 and 2 are in-scope for the implementation. Items 3 and 4 are follow-ons.

---

## Verdict

APPROVED. The feature is well-specified, the design is production-grade, and the QA plan provides complete coverage. Implementation can proceed directly from these artifacts.
