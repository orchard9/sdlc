# Security Audit: Commands Catalog — /docs/commands

## Scope

Three new/modified files:

- `frontend/src/components/docs/commands-data.ts` — static data
- `frontend/src/components/docs/CommandsCatalog.tsx` — React component
- `frontend/src/pages/DocsPage.tsx` — route host

This feature is entirely frontend-only. No backend changes, no new API routes, no auth surface, no user data stored. The security surface is minimal.

---

## Findings

### A1 — XSS via `query` state in empty-state message (FIXED)

**Finding:** The empty-state message renders the search query string inside JSX. If `query` contained a script injection payload, it would need to be properly escaped.

**Analysis:** React's JSX rendering automatically escapes string values rendered as text content (`{query}`). The empty state uses `&ldquo;{query}&rdquo;` which is rendered as a React text node — not `dangerouslySetInnerHTML`. No raw HTML insertion occurs. The `query` state value is always passed through React's built-in escaping before rendering.

**Action:** Accept — no fix required. React's default escaping prevents XSS for text-node rendering.

### A2 — Clipboard API access (ACCEPT)

**Finding:** `CopyButton` calls `navigator.clipboard.writeText(text)` where `text` is `entry.invocation` from the static `COMMANDS` array.

**Analysis:** The clipboard content is sourced entirely from the compile-time static `commands-data.ts` array — no user input, no server data, no dynamic interpolation. There is no path for an attacker to inject content into the clipboard via this feature. The clipboard API is already used throughout the app (in other `CopyButton` uses) and follows the same pattern.

**Action:** Accept.

### A3 — Search input does not need sanitization (ACCEPT)

**Finding:** The `query` state from the search input is used for two purposes: (1) substring matching against static strings, and (2) display in the empty-state message.

**Analysis:** Case (1) uses `.includes()` and `.toLowerCase()` on static strings — no eval, no regex injection surface, no dynamic execution. Case (2) is rendered as a React text node (described in A1). No sanitization is needed.

**Action:** Accept.

### A4 — No new network requests introduced (ACCEPT)

**Finding:** Verify no fetch/XHR calls added.

**Analysis:** `commands-data.ts` exports a plain TypeScript constant — no `fetch`, no `axios`, no dynamic import. `CommandsCatalog.tsx` uses only React hooks (`useState`, `useMemo`) and renders static data. No new network surface.

**Action:** Accept.

### A5 — Static data not user-controlled (ACCEPT)

**Finding:** Verify that the `COMMANDS` array cannot be influenced by URL parameters, query strings, or other runtime inputs.

**Analysis:** The array is a compile-time constant in `commands-data.ts`. It is never augmented at runtime. The only runtime input is the `query` state from the search input, which is used for filtering only — it never modifies the underlying `COMMANDS` data. Accept.

---

## Verdict

**APPROVE.** No meaningful security surface introduced. All findings accepted — no XSS path, no user-controlled data written to DOM as HTML, no new network requests, clipboard access limited to static compile-time data. This feature is a read-only static UI component.
