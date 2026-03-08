# QA Plan: GET /api/git/files endpoint

## Unit Tests

1. **parse_file_entries — ordinary modified (unstaged)**: Input with `1 .M ...` line produces entry with `status: "modified"`, `staged: false`, `unstaged: true`.
2. **parse_file_entries — staged modified**: Input with `1 M. ...` line produces entry with `status: "staged_modified"`, `staged: true`, `unstaged: false`.
3. **parse_file_entries — both staged and unstaged**: Input with `1 MM ...` line produces entry with `staged: true`, `unstaged: true`.
4. **parse_file_entries — staged added**: Input with `1 A. ...` line produces `status: "staged_added"`.
5. **parse_file_entries — deleted**: Input with `1 .D ...` produces `status: "deleted"`.
6. **parse_file_entries — staged deleted**: Input with `1 D. ...` produces `status: "staged_deleted"`.
7. **parse_file_entries — renamed**: Input with `2 R. ...` line produces `status: "staged_renamed"` with correct new path extracted.
8. **parse_file_entries — untracked**: Input with `? new-file.txt` produces `status: "untracked"`.
9. **parse_file_entries — conflicted**: Input with `u UU ...` produces `status: "conflicted"`.
10. **parse_file_entries — empty output**: Empty string produces empty vec.
11. **parse_file_entries — branch headers ignored**: Lines starting with `#` are skipped.

## Integration Verification

12. **Compile check**: `cargo build --all` succeeds with the new endpoint.
13. **Test suite**: `SDLC_NO_NPM=1 cargo test --all` passes, including all new unit tests.
14. **Clippy**: `cargo clippy --all -- -D warnings` produces no warnings.

## Manual Smoke Test (optional)

15. Start the server and `curl http://localhost:7777/api/git/files` — verify JSON response with actual workspace file statuses.
16. `curl http://localhost:7777/api/git/files?include_clean=true` — verify clean tracked files appear.
