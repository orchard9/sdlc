# QA Plan: Orchestrator Actions REST API

## Verification Method

Primary: automated integration tests in `crates/sdlc-server/tests/integration.rs` using `build_router_with_db`.
Secondary: cargo clippy clean build.

All tests run with `SDLC_NO_NPM=1 cargo test --all`.

---

## Q1 — Data layer correctness (sdlc-core unit tests)

**Scope:** `ActionDb::delete` and `ActionDb::update_label_and_recurrence`

| Check | Pass Condition |
|---|---|
| `delete` removes an existing action | `list_all()` returns empty after delete |
| `delete` on non-existent ID succeeds silently | Returns `Ok(())`, no panic |
| `update_label_and_recurrence` with new label | `list_all()` shows updated label, `updated_at` advanced |
| `update_label_and_recurrence` with `recurrence = Some(Some(d))` | `recurrence` field is set |
| `update_label_and_recurrence` with `recurrence = Some(None)` | `recurrence` field is `None` |
| `update_label_and_recurrence` with `label = None, recurrence = None` | No fields changed |
| `update_label_and_recurrence` with unknown ID | Returns `SdlcError::OrchestratorDb("not found")` |

---

## Q2 — GET /api/orchestrator/actions

| Check | Pass Condition |
|---|---|
| Empty DB | 200 with `[]` |
| After one insert | 200 with array of one element |
| Multiple actions | Returns all, sorted newest-first by `created_at` |
| Action JSON has all required fields | `id`, `label`, `tool_name`, `tool_input`, `trigger_type`, `next_tick_at`, `status`, `recurrence_secs`, `created_at`, `updated_at` |
| No `action_db` | 503 Service Unavailable |

---

## Q3 — POST /api/orchestrator/actions

| Check | Pass Condition |
|---|---|
| Valid body | 201 Created, body matches sent fields |
| Returned action visible in GET | 200 with action in list |
| Empty `label` | 400 Bad Request |
| Empty `tool_name` | 400 Bad Request |
| `tool_name` with path traversal (`../evil`) | 400 Bad Request |
| `tool_input` is not an object (e.g. array `[]`) | 400 Bad Request |
| Missing required field `next_tick_at` | 400 (serde deserialization failure) |
| `recurrence_secs` omitted | `recurrence_secs: null` in response |
| `recurrence_secs: 86400` | `recurrence_secs: 86400` in response |
| No `action_db` | 503 Service Unavailable |

---

## Q4 — DELETE /api/orchestrator/actions/{id}

| Check | Pass Condition |
|---|---|
| Delete existing action | 204 No Content; action no longer in GET list |
| Delete non-existent valid UUID | 204 No Content |
| Delete with invalid UUID (`bad`) | 400 Bad Request |
| No `action_db` | 503 Service Unavailable |

---

## Q5 — PATCH /api/orchestrator/actions/{id}

| Check | Pass Condition |
|---|---|
| PATCH `{"label": "renamed"}` | 200; `label` updated in response and in GET |
| PATCH `{"recurrence_secs": 3600}` | 200; `recurrence_secs: 3600` in response |
| PATCH `{"recurrence_secs": null}` | 200; `recurrence_secs: null` in response |
| PATCH `{}` (no fields) | 200; action unchanged |
| PATCH non-existent valid UUID | 404 Not Found |
| PATCH invalid UUID in path | 400 Bad Request |
| PATCH `{"label": ""}` | 400 Bad Request |
| No `action_db` | 503 Service Unavailable |

---

## Q6 — Build quality

| Check | Pass Condition |
|---|---|
| `SDLC_NO_NPM=1 cargo test --all` | All tests pass, zero failures |
| `cargo clippy --all -- -D warnings` | Zero warnings, zero errors |
| No `unwrap()` in library code | Manual scan of `db.rs` changes |
| No `unwrap()` in route handlers | Manual scan of `orchestrator.rs` additions |

---

## Definition of Done

All Q1–Q6 checks pass with zero failures or warnings. The feature is ready to merge when:
- `cargo test --all` is green
- `cargo clippy` is clean
- All integration test cases for the four new endpoints pass
