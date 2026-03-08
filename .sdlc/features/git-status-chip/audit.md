# Security Audit: Git Status Chip

## Scope

Frontend-only feature: a React component + custom hook that polls `GET /api/git/status` and optionally calls `POST /api/git/commit`.

## Findings

### SA-1: API Endpoint Security (Information)
- The chip calls `GET /api/git/status` and `POST /api/git/commit`. These endpoints are defined by the `git-status-api` feature (not yet built).
- **Action**: Accepted. The endpoints will inherit the server's existing auth middleware (tunnel auth with token/cookie gate). No additional auth surface is introduced by this frontend component.

### SA-2: No User Input Injection
- The component renders data from the API response (branch name, counts). No user-supplied input is processed.
- React's JSX escaping prevents XSS from API data.
- **Action**: Accepted. No injection risk.

### SA-3: Commit Without Confirmation
- The commit button fires `POST /api/git/commit` directly without a confirmation dialog. This is by design (spec FR-3).
- **Action**: Accepted. The commit uses a default message and only commits already-staged changes. The git history serves as the undo mechanism, consistent with project ethos ("git is the undo button").

### SA-4: Polling Interval
- 10-second polling is reasonable and does not constitute a DoS risk against the local server.
- Polling pauses when the tab is hidden, preventing unnecessary load.
- **Action**: Accepted. No concern.

### SA-5: Error Handling
- Network errors are caught and surfaced as a grey/disabled state. No sensitive information is leaked in error states.
- `console.warn` is used for commit failures -- acceptable for development diagnostics.
- **Action**: Accepted.

## Verdict

**APPROVED** -- No security concerns. Pure frontend component with no new attack surface. Backend endpoint security is delegated to the server auth middleware.
