# QA Plan: Secrets — POST /api/secrets/envs create-only endpoint

## Scope

Verify that `POST /api/secrets/envs` behaves correctly for all acceptance criteria defined in the spec.

## Test Cases

### TC-1: Empty pairs returns 400

**Setup:** No prior state required; the pairs map is empty in the request body.

**Steps:**
1. Call `POST /api/secrets/envs` with `{ "env": "test", "pairs": {} }`.

**Expected:** HTTP 400 with error message indicating pairs must not be empty.

---

### TC-2: No keys configured returns 400

**Setup:** A fresh sdlc directory with no `keys.yaml` (or an empty keys list).

**Steps:**
1. Call `POST /api/secrets/envs` with `{ "env": "test", "pairs": { "FOO": "bar" } }`.

**Expected:** HTTP 400 with error indicating no keys are configured.

---

### TC-3: Env already exists returns 409

**Setup:**
1. Add a key with `sdlc secrets keys add`.
2. Create `test.age` by calling the endpoint once successfully.

**Steps:**
1. Call `POST /api/secrets/envs` again with `{ "env": "test", "pairs": { "NEW": "val" } }`.

**Expected:** HTTP 409 Conflict.

**Note:** Requires `age` binary installed. Skip if absent.

---

### TC-4: Successful creation returns 201

**Setup:** At least one key configured.

**Steps:**
1. Call `POST /api/secrets/envs` with `{ "env": "staging", "pairs": { "API_URL": "https://api.example.com", "TOKEN": "tok-123" } }`.

**Expected:**
- HTTP 201 Created.
- Response body: `{ "status": "created", "env": "staging", "key_names": ["API_URL", "TOKEN"] }` (order may vary).
- `.sdlc/secrets/envs/staging.age` file exists on disk.

**Note:** Requires `age` binary installed. Skip if absent.

---

### TC-5: GET /api/secrets/envs still works after adding POST

**Steps:**
1. Call `GET /api/secrets/envs`.

**Expected:** HTTP 200 with a JSON array (can be empty).

**Rationale:** Verify route chaining doesn't break the existing GET.

---

### TC-6: Cargo build and clippy pass

**Steps:**
```bash
SDLC_NO_NPM=1 cargo test --all
cargo clippy --all -- -D warnings
```

**Expected:** Zero test failures, zero clippy warnings.

---

## Coverage Matrix

| Acceptance Criteria | Test Case |
|---|---|
| POST creates env, returns 201 | TC-4 |
| POST on existing env returns 409 | TC-3 |
| POST with empty pairs returns 400 | TC-1 |
| POST with no keys returns 400 | TC-2 |
| Route registered (GET still works) | TC-5 |
| Unit tests exist in codebase | TC-6 (cargo test) |
