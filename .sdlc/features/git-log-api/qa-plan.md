# QA Plan: git-log-api

## Test Strategy

Unit tests for the parsing logic; build verification for compilation and route registration.

## Test Cases

### 1. Parser: clean multi-commit output
- **Input:** Synthetic git log output with 3 commits using separator format
- **Expected:** Returns `Vec<CommitEntry>` with 3 entries, all fields correctly populated

### 2. Parser: empty output (no commits)
- **Input:** Empty string
- **Expected:** Returns empty Vec

### 3. Parser: single commit
- **Input:** One commit entry
- **Expected:** Returns Vec with 1 entry, all fields correct

### 4. Parser: multi-line commit body
- **Input:** Commit with subject and multi-line body
- **Expected:** `subject` contains first line only, `body` contains remaining lines

### 5. Parser: special characters in message
- **Input:** Commit message with quotes, angle brackets, unicode
- **Expected:** All characters preserved correctly

### 6. Pagination clamping
- **Input:** `per_page=200`
- **Expected:** Clamped to 100

### 7. Pagination defaults
- **Input:** No query parameters
- **Expected:** `page=1`, `per_page=25`

### 8. Build verification
- **Verify:** `SDLC_NO_NPM=1 cargo build --all` compiles without errors
- **Verify:** `SDLC_NO_NPM=1 cargo test --all` passes including new tests

## Pass Criteria

- All unit tests pass
- Crate compiles without warnings under `cargo clippy`
- Route is registered and reachable
