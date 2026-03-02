# Security Audit: Orchestrator Actions REST API

## Feature
`orchestrator-actions-routes`

## Audit Date
2026-03-02

## Surface Area

This feature exposes four HTTP endpoints under `/api/orchestrator/actions`:

| Endpoint | Method | Purpose |
|---|---|---|
| `/api/orchestrator/actions` | GET | List all scheduled actions |
| `/api/orchestrator/actions` | POST | Create a new scheduled action |
| `/api/orchestrator/actions/{id}` | DELETE | Delete an action by UUID |
| `/api/orchestrator/actions/{id}` | PATCH | Update label and/or recurrence |

It also adds two new methods to `ActionDb` (`delete`, `update_label_and_recurrence`) and 12 integration tests.

---

## Security Findings

### 1. Input Validation — Path Traversal in `tool_name` [ADDRESSED]

**Risk:** The `create_action` endpoint accepts a `tool_name` field that is used as a tool slug under `.sdlc/tools/<name>/`. A malicious value like `../secrets` could attempt filesystem escape.

**Finding:** The handler calls `sdlc_core::paths::validate_slug(&body.tool_name)` before any DB write. This rejects values containing `/`, `..`, or other non-slug characters, returning `400 Bad Request`.

**Status:** Mitigated — test `create_action_invalid_tool_name` confirms `"../evil"` → 400.

---

### 2. UUID Validation on Path Parameters [ADDRESSED]

**Risk:** Accepting arbitrary strings as `{id}` path parameters could cause panics or unexpected behavior.

**Finding:** Both `delete_action` and `patch_action` parse `id` via `id.parse::<uuid::Uuid>()` and return `400 Bad Request` on failure. The string is not used in any filesystem or SQL operation — only as a lookup key in `list_all()`.

**Status:** Mitigated — test `delete_action_invalid_uuid` confirms non-UUID → 400.

---

### 3. Authorization Boundary [ACCEPTABLE]

**Risk:** These endpoints expose CRUD over orchestration actions (scheduled tool runs). An unauthorized caller could create, delete, or modify action schedules.

**Finding:** The `sdlc-server` auth layer (`crates/sdlc-server/src/auth.rs`) gates all `/api/*` routes behind the tunnel auth middleware. Local requests (same machine, no tunnel token) are allowed by design. Remote tunnel access requires a token. This is consistent with all other `/api/orchestrator/*` endpoints and all other admin-tier routes in this server.

**Status:** Acceptable given the local-first, developer-tool security model. The auth boundary is identical to adjacent endpoints.

---

### 4. `tool_input` JSON Object Validation [ADDRESSED]

**Risk:** If `tool_input` is not an object (e.g. a string or array), downstream tool execution could behave unexpectedly.

**Finding:** `create_action` checks `body.tool_input.is_object()` and returns `400 Bad Request` if the input is not a JSON object.

**Status:** Mitigated.

---

### 5. Label Content Validation [LOW RISK / ACCEPTABLE]

**Risk:** The `label` field is a free-form string. It is stored in the DB and displayed in the UI but is never used in shell commands or filesystem paths.

**Finding:** Only non-empty validation is applied. No length limit, no character allowlist. Given that `label` is display-only, this is acceptable. A future hardening pass could add a max length (e.g. 256 chars), tracked as a non-blocking observation.

**Status:** Acceptable for current scope. No immediate risk.

---

### 6. `recurrence_secs` Integer Range [ACCEPTABLE]

**Risk:** An extremely large `recurrence_secs` value could cause integer overflow.

**Finding:** `recurrence_secs` is deserialized as `u64` (via `MaybeAbsent<u64>`) and then passed to `std::time::Duration::from_secs(u64)`. The maximum u64 is ~585 billion years; `Duration` accepts this without overflow. The value is stored as seconds in `Action.recurrence` and surfaced back in the API. No scheduler arithmetic is performed at ingestion time.

**Status:** No risk.

---

### 7. Error Message Information Disclosure [LOW RISK]

**Finding:** The `patch_action` handler uses `e.to_string().contains("not found")` to distinguish 404 from 500 errors. The raw error string (e.g. `"action not found: <uuid>"`) is returned in the 404 JSON body. This exposes the internal UUID format in error messages, but since the UUID is the client-supplied value, this is not a meaningful disclosure.

**Status:** Acceptable.

---

### 8. Concurrent Write Safety [ACCEPTABLE]

**Risk:** Two concurrent PATCH or DELETE requests for the same action ID could conflict.

**Finding:** redb uses write-transaction locking — concurrent writes are serialized at the DB level. `list_all()` is called inside the write operation block, and the key is computed from the found action's trigger timestamp. If two concurrent patches run: one will succeed; the other will find the updated value (since it calls `list_all()` anew). The result is deterministic — last write wins. For a developer tool with low concurrency, this is acceptable.

**Status:** Acceptable for current scale.

---

## Summary

No blocking security issues found. Two medium-priority items are addressed by existing validation (path traversal, UUID validation). The auth boundary is consistent with the rest of the server. No new attack surfaces beyond what existing orchestrator routes already expose.

## Verdict

**APPROVED** — ready to advance to QA.
