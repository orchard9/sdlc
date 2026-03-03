# Security Audit: Fix Changelog Polling Infinite Loop on null lastVisitAt

## Scope

Single-file frontend hook change: `frontend/src/hooks/useChangelog.ts`. The only modification is replacing an inline `Date.now()` expression with `useMemo`.

## Security Surface

**Attack vectors considered:**

1. **Client-side data exposure** — `since` is a timestamp computed from `Date.now()` or `localStorage`. Neither is sensitive. No PII, credentials, or secrets involved.

2. **localStorage manipulation** — `localStorage.getItem(STORAGE_KEY)` was already read before this fix. The `useMemo` change does not alter how the value is read or validated. An attacker who can write to localStorage could supply a crafted `since` value, but this was already true before the fix. The server-side changelog endpoint is responsible for validating query parameters.

3. **Request flooding (was the bug)** — The fix eliminates the unintentional DoS-like behaviour against the local server. Post-fix, the hook makes one request on mount and one per SSE event — the intended rate.

4. **XSS** — No DOM interaction, no `innerHTML`, no `dangerouslySetInnerHTML`. Not applicable.

5. **Dependency chain** — `useMemo` is a core React hook. No new third-party dependencies introduced.

## Findings

None. The change has no meaningful security surface beyond what existed before. The fix strictly reduces the attack surface by eliminating the unintentional request storm.

## Verdict: APPROVE
