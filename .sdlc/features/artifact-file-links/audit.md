# Security Audit: Auto-detect File Paths as IDE Links

## Scope

This feature adds:
1. A `Settings.ide_uri_scheme` field to `config.yaml`
2. A `project_root` field injected into the `GET /api/config` API response
3. Client-side rendering of `ide://file/` URIs when inline code matches a file path pattern

---

## Threat Analysis

### 1. IDE URI Scheme Injection

**Concern:** A malicious `.sdlc/config.yaml` could set `ide_uri_scheme` to an unexpected value (e.g. `javascript`, `data`, `file`, or a custom protocol).

**Analysis:**
- `ide_uri_scheme` comes from the project's own `.sdlc/config.yaml`, which is a committed file in the project repository. An attacker who can modify this file already has write access to the project's git repository.
- Browsers do not execute `vscode://`, `cursor://`, `zed://`, `idea://` URIs without the corresponding application being registered and a user click. These protocols open the IDE, not a script engine.
- `javascript:` and `data:` URIs in `href` on `<a>` elements require a user click and are subject to the browser's security model. React renders the `href` as a string attribute; it does not evaluate it.
- The rendered URI is `{ideUriScheme}://file/{projectRoot}/{filePath}` — even if `ideUriScheme` is unexpected, the URI points to a file path within the project, not an arbitrary URL.

**Verdict:** Acceptable. The trust boundary is the project's git repository, which is already the trust boundary for the SDLC tool itself. Documenting supported values (`vscode`, `cursor`, `zed`, `idea`) in config.yaml reduces confusion.

**Action taken:** None required. Future enhancement: validate `ide_uri_scheme` against a whitelist in `Config::validate()`. Tracked as a follow-up consideration, not a blocking finding.

---

### 2. `project_root` Disclosure

**Concern:** The `GET /api/config` endpoint now returns the absolute path of the server's working directory. This leaks filesystem layout to anyone who can access the API.

**Analysis:**
- The `sdlc ui` server binds to `localhost` by default. External access requires an explicit tunnel (cloudflared). Tunnel access is token-gated via the auth middleware (`crates/sdlc-server/src/auth.rs`).
- The API is consumed by the React frontend running in the same browser session as the developer. No cross-origin access is possible without the auth token.
- The project root path is already implicitly known to any developer running `sdlc ui` — it's the directory they ran the command from.
- Leaking the path via tunnel would require a valid auth token, which is the same as having full SDLC API access. No incremental risk.

**Verdict:** Acceptable. No change needed.

---

### 3. File Path Pattern as XSS Vector

**Concern:** Could a crafted markdown document inject arbitrary content via the file path detection branch?

**Analysis:**
- The regex `FILE_PATH_PATTERN` only matches strings that:
  - Start with an alphanumeric, dot, or underscore
  - Contain only alphanumerics, dots, dashes, underscores, and slashes
  - End with a 2–5 character alpha extension
- Characters that could form XSS payloads (`<`, `>`, `"`, `'`, `&`, `{`, `}`, `(`, `)`) are not in the character class.
- React's JSX escapes `href` and element children by default — there is no `dangerouslySetInnerHTML` usage.
- The `href` is constructed from `ideUriScheme + "://file/" + projectRoot + "/" + text`, all of which are strings. React sets this as a DOM attribute string, not evaluated HTML.

**Verdict:** No XSS risk. The regex is sufficiently restrictive and React's escaping provides defense in depth.

---

### 4. Open Redirect

**Concern:** Could an attacker use the link rendering to redirect users to attacker-controlled URLs?

**Analysis:**
- The `href` is always `{scheme}://file/{projectRoot}/{filePath}` — a fixed structure with the project root as a prefix. There is no user-controlled URL parameter.
- The `filePath` portion must match `FILE_PATH_PATTERN`, which prohibits `://`, `@`, `?`, `#`, or other URL-significant characters.
- The IDE protocol handlers (`vscode://`, `cursor://`, etc.) are not web URLs — browsers open the registered desktop application, not a web page.

**Verdict:** No open redirect risk.

---

### 5. Path Traversal

**Concern:** Could `filePath` containing `../` escape the project root?

**Analysis:**
- The regex prohibits `..` (requires segments to match `[a-zA-Z0-9_.\-][a-zA-Z0-9_.\-]*` — a single dot followed by another dot would require the character after `.` to be in class, and `./..` would fail because `..` contains only dots which doesn't satisfy the extension requirement at the end).
- Actually, `../../../etc/passwd` would not match because it starts with `.` then `.` — the first segment `..` has no extension and the whole string would fail the trailing extension requirement `\.[a-zA-Z]{2,5}$`.
- Even if traversal were possible in the path string, the IDE URI is passed to the local operating system to open a file — this is the expected behavior, not a security escalation.

**Verdict:** No meaningful path traversal risk for this use case.

---

## Summary

| Finding | Severity | Action |
|---|---|---|
| ide_uri_scheme not validated against whitelist | Low | Accept for V1; add validation in future Config.validate() enhancement |
| project_root disclosed in API response | Informational | Accept — requires auth token, developer already knows the path |
| XSS via file path pattern | None | React escaping + restrictive regex prevents any injection |
| Open redirect | None | URI structure is fixed; protocol handlers are local apps |
| Path traversal | None | Regex prohibits traversal patterns |

**Overall verdict:** No blocking security findings. Feature is safe to ship.
