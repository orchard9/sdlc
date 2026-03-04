# Security Audit: Scrapbook renders *-mockup.html artifacts in an inline iframe

## Surface Area

This feature adds a browser iframe to render HTML scrapbook artifacts in the
`WorkspacePanel` component. HTML artifacts are already rendered by `ArtifactContent`
(introduced earlier) — this feature adjusts heights and adds visual indicators.

The iframe rendering of HTML content is the security surface to audit.

---

## Finding 1: iframe sandbox attribute — PASS

**The `iframe` in `ArtifactContent` uses `sandbox="allow-scripts"`.**

This is the correct minimum sandbox for rendering HTML mockups that may use Tailwind
CDN or simple JavaScript. The sandbox prevents:
- `allow-forms` — no form submission
- `allow-same-origin` — no access to the parent page's cookies, localStorage, or DOM
- `allow-top-navigation` — no redirecting the top frame
- `allow-popups` — no new windows

The `allow-scripts` permission is required for Tailwind CDN and any state-toggle JS
in mockups. Without it, Tailwind utility classes would not apply and the mockup would
render unstyled.

**Action:** Accept — appropriate sandbox for self-contained design mockups.

---

## Finding 2: Content is `srcDoc`, not a URL — PASS

**The iframe uses `srcDoc={content}` not `src="url"`.**

This means the rendered content is the literal string of the HTML artifact, not a
fetched URL. There is no SSRF risk and no opportunity for URL injection. The content
originates from the `.sdlc/roadmap/<slug>/` directory on disk — only agents and CLI
users with filesystem access can write artifacts.

**Action:** Accept — no injection risk.

---

## Finding 3: HTML content is agent-authored, not user-typed — PASS

Scrapbook artifacts are written by agents using `sdlc ponder capture` (CLI) or
`POST /api/roadmap/:slug/capture` (server). The server endpoint is authenticated via
`auth.rs` (token/cookie gate with local bypass). No untrusted external user can inject
content into the scrapbook without authentication.

**Action:** Accept — trust boundary is adequate.

---

## Finding 4: No `allow-same-origin` means no cookie/storage exfil — PASS

Without `allow-same-origin`, the sandboxed iframe content cannot access
`document.cookie`, `localStorage`, `sessionStorage`, or any other origin-scoped
browser data. Even if a malicious HTML file were captured, it cannot exfiltrate
session tokens or application state.

**Action:** Accept — defense-in-depth is in place.

---

## Finding 5: External scripts in mockups (Tailwind CDN) — ACCEPTABLE RISK

The design artifact protocol encourages `<script src="https://cdn.tailwindcss.com">`.
Because `allow-same-origin` is not set, the Tailwind script runs in a sandboxed null
origin and cannot access parent page data. The only risk is CDN availability (not a
security risk) or CDN compromise (supply chain risk, not specific to this feature).

**Action:** Accept — risk is the same as any CDN usage and bounded by sandbox.

---

## Verdict

**No security findings requiring remediation.** The iframe sandbox is correctly
configured for the use case. Content origin is authenticated. No same-origin access.
The change is safe to merge.
