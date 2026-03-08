# Security Audit: git-status-api

## Scope

New `GET /api/git/status` endpoint that shells out to `git` to collect repository state.

## Findings

### 1. Command Injection — NOT VULNERABLE

The endpoint passes no user input to git commands. All arguments are hardcoded string literals (`"rev-parse"`, `"--git-dir"`, `"status"`, `"--porcelain=v2"`, `"--branch"`). The `current_dir` is set to `app.root` which is configured at server startup, not from request parameters.

**Action:** No fix needed. No user-controlled input reaches the command execution.

### 2. Path Traversal — NOT VULNERABLE

The endpoint uses `app.root` (set at server init) as the working directory. There are no query parameters or path parameters that could influence which directory is inspected.

**Action:** No fix needed.

### 3. Information Disclosure — LOW RISK

The endpoint exposes: branch name, file counts (dirty/staged/untracked/conflict), and ahead/behind tracking. It does NOT expose file names, file contents, or diff content. This is equivalent to what a user would see in a status bar.

The endpoint is behind the standard auth middleware (tunnel token/cookie gate), so unauthenticated users cannot access it.

**Action:** Accepted — information exposed is minimal and behind auth.

### 4. Denial of Service — LOW RISK

`git status` can be slow on very large repositories (100k+ files). The endpoint uses `spawn_blocking` which prevents blocking the async runtime, but a slow git command could occupy a blocking thread.

**Action:** Accepted — this is the same risk as any other endpoint that calls git (e.g., the existing git-sync pattern in fleet mode). The blocking thread pool has bounded capacity by default.

### 5. Subprocess Error Handling — ACCEPTABLE

Git subprocess stderr output is included in error messages via `String::from_utf8_lossy`. This could theoretically leak filesystem paths in error responses. However, this only occurs when git itself fails (misconfigured repo, corrupted objects), which is an exceptional condition.

**Action:** Accepted — error messages from git are informational, not sensitive, and the endpoint is behind auth.

## Verdict

**Approved.** No security vulnerabilities found. The endpoint is read-only, takes no user input, shells out with hardcoded arguments only, and is protected by auth middleware.
