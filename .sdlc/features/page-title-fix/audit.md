# Security Audit: page-title-fix

## Change Summary

Server-side injection of a project name string into the HTML `<title>` tag when serving `index.html` from the embedded frontend assets.

## Attack Surface Analysis

### XSS via project name injection

**Finding:** The `inject_title` function inserts `state.project` directly into HTML between `<title>` and `</title>`. If a malicious project name contains `</title><script>...`, it would break out of the title context.

**Assessment:** The project name in `state.yaml` is set by the project owner (the same person running `sdlc ui`). It is not user-provided input from an HTTP request. An attacker cannot inject an arbitrary project name via the HTTP interface — they would need filesystem write access to `.sdlc/state.yaml`, which already implies full system compromise.

**Verdict:** Not a meaningful attack vector. The input origin is trusted (local filesystem, owner-controlled).

**However** — defense in depth: a simple HTML-escape on the title value would eliminate the theoretical issue at zero cost. This is tracked as a non-blocking improvement.

**Action:** Accept as-is for this iteration. Track as a task for hardening.

### Information disclosure

**Finding:** The page title exposes the project name to anyone who can load the UI.

**Assessment:** The project name is already exposed via `/api/state` (the `project` field). The title provides no additional information beyond what the API already returns. Auth middleware (token gate) protects both the API and the HTML page equally.

**Verdict:** No issue.

### Path traversal / YAML injection

**Finding:** `State::load(root)` reads `.sdlc/state.yaml` from `app.root`. Could a crafted root path cause unexpected file reads?

**Assessment:** `app.root` is set at server startup from the command-line argument and never changes at runtime. No HTTP parameter influences it. No issue.

**Verdict:** No issue.

### Denial of service via large project name

**Finding:** A very large project name would be returned in every `index.html` response.

**Assessment:** The project name in `state.yaml` is bounded by normal human usage. `sdlc init` and the CLI impose no specific limit, but the YAML file is local and owner-controlled. Even a 10KB project name adds negligible overhead to an HTTP response.

**Verdict:** No issue.

## Findings Summary

| Finding | Severity | Action |
|---------|----------|--------|
| XSS via project name in title tag (theoretical) | Low | Accept — owner-controlled input; track for HTML escaping hardening |
| Information disclosure | None | No action |
| Path traversal | None | No action |
| DoS via large project name | None | No action |

## Improvement Task

Track HTML escaping of the project name in the title for defense-in-depth:

> Escape `<`, `>`, `&`, `"`, `'` in `compute_title` output before inserting into HTML.

This is non-blocking for the current feature — the input is owner-controlled, not user-provided.

## Verdict: Approved

No security issues that require blocking action before merge. One low-severity defense-in-depth improvement tracked for follow-on.
