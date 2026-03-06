# Security Audit: hub-create-repo-ui

## Surface

Frontend component that calls `POST /api/hub/create-repo` and displays the returned `push_url` (which contains an embedded admin token) to the user.

## Findings

### F1 — Push URL token displayed in plaintext [ACCEPTED]

**Finding:** The `push_url` returned from the API contains the Gitea admin token in HTTP basic auth. It is displayed in a `<code>` block and written to the clipboard.

**Risk:** A user who sees the screen can capture the admin token. In a multi-user scenario this would be a problem. For this single-operator tool it is acceptable — the authenticated hub user is the operator.

**Accepted because:** Same rationale as backend audit F1. The UI is behind Google OAuth. The copy mechanism is clipboard-only; the token is not sent to any third-party service.

**Action:** None. Accepted.

### F2 — No XSS surface [PASS]

`push_url` and `repo_slug` from the API response are used as React children in `<code>` elements — React escapes these automatically. No `dangerouslySetInnerHTML`. No user input is reflected into the DOM without React's escaping.

### F3 — Clipboard API graceful failure [PASS]

`navigator.clipboard.writeText(text).catch(() => {})` — failure (permission denied) is silenced. The code text is still visible in the UI for manual copy. No security concern.

### F4 — Client-side name validation [PASS]

The regex guard prevents submitting names with spaces, special chars, or uppercase letters. This is defense-in-depth — the server validates independently. No injection surface.

## Verdict

APPROVED. One accepted finding (token in push_url display) with same documented rationale as backend audit.
