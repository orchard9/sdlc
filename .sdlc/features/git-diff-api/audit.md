# Security Audit: GET /api/git/diff endpoint

## Scope

The `GET /api/git/diff` endpoint accepts a user-supplied `path` query parameter and passes it to git commands. The primary attack surface is path traversal and command injection.

## Findings

### F1: Path Traversal — MITIGATED
**Risk:** An attacker could supply `../../etc/passwd` to read files outside the project root.
**Mitigation:** `validate_diff_path` checks every path component using `std::path::Component::ParentDir`, rejecting any path containing `..`. This is robust — it uses the standard library's path parsing rather than string matching, so it handles normalized and edge-case paths correctly.
**Action:** Accepted as mitigated.

### F2: Command Injection via Path Parameter — MITIGATED
**Risk:** The `path` parameter is passed as an argument to `Command::new("git").args(...)`. Could a crafted path inject shell commands?
**Mitigation:** Rust's `std::process::Command` does not invoke a shell — arguments are passed directly to the process via `execvp`. There is no shell interpretation of special characters (`; | & $`). The path is passed as a separate argument after `--`, which prevents git from interpreting it as a flag.
**Action:** Accepted as mitigated.

### F3: File Content Disclosure — ACCEPTABLE RISK
**Risk:** The endpoint returns file content (as diff output) for any file in the project root.
**Assessment:** This is by design — the endpoint serves the same data available via `git diff` in the terminal. The server already has full filesystem access to the project root. The path traversal check (F1) prevents reading outside the project. This is consistent with the git-status endpoint's design.
**Action:** Accepted by design.

### F4: Untracked File Full Content — ACCEPTABLE RISK
**Risk:** For untracked files, `git diff --no-index /dev/null <path>` returns the entire file content.
**Assessment:** Consistent with design. The user is expected to see the full content of new files in a diff viewer. The path validation ensures only files within the project root are accessible.
**Action:** Accepted by design.

### F5: Denial of Service via Large Files — LOW RISK
**Risk:** Diffing a very large file could consume memory and time.
**Assessment:** Git handles large files reasonably well. The endpoint operates on a single file at a time. No amplification vector exists. In the worst case, a single request blocks one thread momentarily.
**Action:** Accepted as low risk. Could add a size limit in a future iteration if needed.

## Verdict

No unmitigated security issues found. Path traversal and command injection are properly handled. The endpoint's read-only nature and project-scoped access limit the blast radius.
