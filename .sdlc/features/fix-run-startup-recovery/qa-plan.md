# QA Plan: Server Startup Marks Orphaned Runs Failed

## Test Cases

### TC-1: Orphaned run gets `status = "failed"` on startup

**Type:** Unit test (automated)
**Location:** `crates/sdlc-server/src/state.rs` `#[cfg(test)]`

**Setup:**
- Write a `RunRecord` JSON with `status = "running"` to a temp `.sdlc/.runs/` dir.

**Steps:**
1. Call `load_run_history(temp_dir)`.

**Expected:**
- Returned `Vec<RunRecord>` contains a record with `status == "failed"`.
- `error` field is `Some("server restarted")`.
- `completed_at` is `Some(<ISO8601 timestamp>)`.

---

### TC-2: Orphaned run persisted to disk with `status = "failed"`

**Type:** Unit test (automated)

**Setup:** Same as TC-1.

**Steps:**
1. Call `load_run_history(temp_dir)`.
2. Read the JSON file back from disk.

**Expected:**
- Deserialized record has `status == "failed"` and `error == "server restarted"`.

---

### TC-3: Non-running records are unchanged

**Type:** Unit test (automated)

**Setup:**
- Write two `RunRecord` JSONs: one with `status = "completed"`, one with
  `status = "stopped"`.

**Steps:**
1. Call `load_run_history(temp_dir)`.

**Expected:**
- Both returned records are unchanged (status, error fields as written).

---

### TC-4: `cargo test --all` passes with `SDLC_NO_NPM=1`

**Type:** Build gate (CI)

```bash
SDLC_NO_NPM=1 cargo test --all
```

**Expected:** Exit code 0, all tests pass.

---

### TC-5: `cargo clippy --all -- -D warnings` passes

**Type:** Lint gate (CI)

```bash
cargo clippy --all -- -D warnings
```

**Expected:** Exit code 0, zero warnings.

---

## Pass Criteria

All five test cases must pass for the QA plan to be approved.
