# QA Plan: GET /api/git/diff endpoint

## Test Strategy

Unit tests cover the pure logic (path validation, status parsing, binary detection). Integration testing validates the full endpoint against a real git repository.

## Unit Tests

### Path Validation
- `validate_path_rejects_dotdot`: Paths containing `..` return `invalid_path` error
- `validate_path_accepts_normal`: Normal relative paths like `src/main.rs` pass validation
- `validate_path_accepts_nested`: Deeply nested paths like `a/b/c/d.rs` pass validation

### File Status Parsing
- `parse_status_modified`: Porcelain v2 `1 .M` output maps to `status: "modified"`
- `parse_status_added`: Porcelain v2 `1 A.` output maps to `status: "added"`, `is_new: true`
- `parse_status_deleted`: Porcelain v2 `1 .D` output maps to `status: "deleted"`, `is_deleted: true`
- `parse_status_renamed`: Porcelain v2 `2 R.` output maps to `status: "renamed"`
- `parse_status_untracked`: Porcelain v2 `?` output maps to `status: "untracked"`, `is_new: true`
- `parse_status_empty`: No output means file is unchanged

### Binary Detection
- `detect_binary_true`: Diff output starting with `Binary files` sets `is_binary: true`
- `detect_binary_false`: Normal diff output sets `is_binary: false`

### DiffResult Serialization
- `diff_result_serializes_correctly`: Verify JSON output matches expected field names and types

## Integration Tests (manual verification)

1. Start the server with `cargo run -p sdlc-server`
2. Modify a file, then `curl http://localhost:7777/api/git/diff?path=<file>` — expect a valid diff
3. Stage a file, then `curl ...?path=<file>&staged=true` — expect staged diff
4. Create a new untracked file, then `curl ...?path=<file>` — expect full-content diff with `"untracked"` status
5. `curl ...` without `path` — expect 400
6. `curl ...?path=../../../etc/passwd` — expect 400 `invalid_path`
7. `curl ...?path=nonexistent.rs` — expect 404

## Pass Criteria

- All unit tests pass (`cargo test -p sdlc-server`)
- No `unwrap()` calls in production code
- Endpoint responds in under 500ms for typical files
- All error cases return appropriate HTTP status codes
