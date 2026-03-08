# QA Plan: git-status-api

## Test Strategy

### Unit Tests

1. **Porcelain v2 parsing — clean repo**: Feed a porcelain output with only branch headers and no changed entries. Assert all counts are 0, severity is "green", summary is "clean".

2. **Porcelain v2 parsing — dirty files**: Feed output with ordinary changed entries (lines starting with `1`). Assert `dirty_count` and `staged_count` are correct based on XY flags.

3. **Porcelain v2 parsing — untracked files**: Feed output with `?` lines. Assert `untracked_count` matches.

4. **Porcelain v2 parsing — conflicts**: Feed output with `u` (unmerged) lines. Assert `has_conflicts == true` and `conflict_count` matches.

5. **Porcelain v2 parsing — ahead/behind**: Feed output with `# branch.ab +3 -2`. Assert `ahead == 3`, `behind == 2`.

6. **Porcelain v2 parsing — detached HEAD**: Feed output with `# branch.head (detached)`. Assert branch field handles this gracefully.

7. **Severity — red (conflicts)**: `has_conflicts = true` -> severity is "red".

8. **Severity — red (far behind)**: `behind = 11` -> severity is "red".

9. **Severity — yellow (dirty)**: `dirty_count = 1`, no conflicts, behind = 0 -> severity is "yellow".

10. **Severity — yellow (behind)**: `behind = 1` -> severity is "yellow".

11. **Severity — green**: All counts zero, no conflicts -> severity is "green".

12. **Summary generation**: Verify summary includes all non-zero fields joined by commas, or "clean" when all zero.

### Integration Tests

13. **Endpoint returns valid JSON**: `GET /api/git/status` on this project's own repo returns 200 with all expected fields present and correct types.

14. **Non-git directory**: `GET /api/git/status` with a temp non-git directory returns `{ "error": "not_a_git_repo" }`.

### Manual Verification

15. **Curl smoke test**: After implementation, `curl http://localhost:7777/api/git/status | jq .` returns expected shape with real data from the sdlc repo.

## Pass Criteria

- All unit tests pass.
- Integration test against a real git repo passes.
- No `unwrap()` in the implementation.
- Endpoint responds in under 500ms.
