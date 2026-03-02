# Security Audit: FeedbackThread UI

## Scope

Frontend-only change. No new Rust server code, no new API endpoints, no authentication changes. The audit focuses on:

1. XSS / injection risks in new React components
2. API call surface and request handling
3. User-controlled data rendering

---

## Finding 1: Thread body and comment body rendered as plain text — PASS

**Location:** `CoreElement.tsx` (body), `CommentCard.tsx` (comment body)

**Observation:** Both render user-controlled content using `<pre className="whitespace-pre-wrap">` with React's default text rendering (not `dangerouslySetInnerHTML`). React escapes all special characters by default.

**Risk:** None. No XSS vector.

---

## Finding 2: Thread title rendered as plain text — PASS

**Location:** `ThreadDetailPane.tsx`, `ThreadListPane.tsx`

**Observation:** `{detail.title}` and `{thread.title}` rendered as React text nodes. Escaped by React.

**Risk:** None.

---

## Finding 3: Author field rendered without sanitization — ACCEPTABLE

**Location:** `CommentCard.tsx` — `{comment.author}` rendered as text

**Observation:** Author is a user-controlled string from the API. Rendered as a plain text node via React. No HTML interpretation possible.

**Risk:** None for XSS. An adversarial author value (e.g., a very long string) could affect layout, but this is a cosmetic concern, not a security issue.

---

## Finding 4: `encodeURIComponent` used on all URL-embedded slugs — PASS

**Location:** `api/client.ts` — all thread API calls use `encodeURIComponent(slug)`

**Observation:** Slug values are properly encoded before inclusion in URL paths. No path traversal risk.

**Risk:** None.

---

## Finding 5: New modal does not prevent focus escaping to background — LOW / ACCEPTABLE

**Location:** `NewThreadModal.tsx`

**Observation:** The modal uses a fixed overlay with `z-50`. It does not implement a full focus trap (no `aria-modal`, no tab cycle management). A user could tab past the modal and interact with background elements.

**Risk:** Accessibility concern (WCAG 2.4.3), not a security risk. Consistent with other modals in the codebase (e.g., `CreateToolModal`). Acceptable for V1.

**Action:** Track as a follow-on accessibility task.

---

## Finding 6: No input length validation on compose textarea — LOW / ACCEPTABLE

**Location:** `ThreadDetailPane.tsx` — compose textarea

**Observation:** No `maxLength` attribute on the comment textarea. The server is responsible for enforcing limits. The UI does not pre-validate.

**Risk:** A user could submit a very large payload. This is a server-side concern (handled by `feedback-thread-core`). No client-side security risk.

**Action:** No change needed — server enforces.

---

## Finding 7: Thread slug comes from API response on create — PASS

**Location:** `ThreadsPage.tsx` — `handleCreateThread` navigates to `newThread.slug`

**Observation:** The slug used for navigation comes from the API response (not from user input directly). The API generates the slug server-side.

**Risk:** None — slug is not constructed from untrusted user input.

---

## Summary

| # | Finding | Severity | Action |
|---|---------|----------|--------|
| 1 | Body/comment rendered as plain text | None | — |
| 2 | Title rendered as plain text | None | — |
| 3 | Author field rendered without sanitization | Cosmetic | — |
| 4 | encodeURIComponent used on all slug params | None | — |
| 5 | Modal lacks focus trap | Low / A11y | Follow-on task |
| 6 | No textarea maxLength | Low | Server handles |
| 7 | Slug from server response | None | — |

No blocking security issues. The feature has no meaningful new attack surface — it is a read/write UI over an API that the existing auth middleware already protects.

## Verdict

**APPROVED.** No security regressions introduced. One follow-on accessibility task noted (focus trap in NewThreadModal).
