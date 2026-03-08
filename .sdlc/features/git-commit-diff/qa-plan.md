# QA Plan: Commit-specific file viewing and diff display

## Backend Tests

### API endpoint validation
1. `GET /api/git/show/<valid-sha>` returns 200 with correct commit metadata, files array, and diff string
2. `GET /api/git/show/ZZZZ` (non-hex) returns 400 with `invalid_sha` error
3. `GET /api/git/show/ab` (too short, <4 chars) returns 400
4. `GET /api/git/show/<nonexistent-40-char-hex>` returns 404 with `commit_not_found`
5. Response `files` array contains correct `path`, `status`, `additions`, `deletions` for each changed file
6. Diff output is truncated at 100KB with `truncated: true` flag

### Unit tests
7. SHA validation function accepts valid hex strings 4-40 chars
8. SHA validation function rejects non-hex, too short, too long
9. Commit metadata parsing extracts sha, author, date, message correctly
10. File stat parsing handles added, modified, deleted, renamed files
11. Diff truncation kicks in at 100KB boundary

## Frontend Tests

### CommitDetail component
12. Renders file list with correct addition/deletion counts
13. Renders diff with proper line coloring (additions green, deletions red, hunks blue)
14. Shows loading state while fetching
15. Shows error state on fetch failure
16. Shows truncation notice when `truncated: true`

## Integration

17. Build compiles without errors: `SDLC_NO_NPM=1 cargo build --all`
18. All tests pass: `SDLC_NO_NPM=1 cargo test --all`
19. Clippy clean: `cargo clippy --all -- -D warnings`
