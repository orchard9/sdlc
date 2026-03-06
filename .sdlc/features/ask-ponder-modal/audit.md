# Security Audit: ask-ponder-modal

## Surface

Pure frontend change. No new backend endpoints, no new API keys, no new data stored client-side beyond transient React state. The only server calls are to existing authenticated endpoints.

## Findings

### 1. API calls use existing auth — no bypass (PASS)

`api.answerAma`, `api.createAmaThread`, `api.addAmaThreadTurn` all go through the existing `request()` wrapper in `client.ts`, which inherits cookies/session from the browser. No credentials handled in the modal code.

### 2. Question text — user-controlled input to API (PASS)

The `question` textarea value is sent directly to `api.answerAma`. The backend is responsible for sanitizing and using it safely. The frontend does not attempt any sanitization (none needed — it's a trusted API call). No XSS risk from the textarea itself; React escapes values by default.

### 3. Thread ID generation via `toThreadId` (LOW — no action)

`toThreadId` generates a slug from the question text. It strips non-alphanumeric characters (`replace(/[^a-z0-9\s]/g, '')`), lowercases, and truncates to 48 chars. The resulting string is used as a thread ID in a URL segment: `api.createAmaThread(threadId, ...)` → `DELETE /api/tools/ama/threads/:id`. The slug cannot contain path traversal characters (slashes, dots) due to the regex. No injection risk.

### 4. EventSource stream — no script execution risk (PASS)

The streamed text from `AmaAnswerPanel`'s EventSource is rendered via `<ReactMarkdown>`. ReactMarkdown with `remark-gfm` does not allow raw HTML by default — `allowDangerousHtml` is not set. Streamed content is safe to render.

### 5. `navigate()` call with user-derived thread ID (PASS)

After creating a thread, the modal navigates to `/threads/${threadId}` where `threadId` is the locally generated slug from `toThreadId(question)`. Since `toThreadId` strips all non-alphanumeric characters (regex `[^a-z0-9\s]` removes `/`, `.`, `..`, etc.), no path traversal is possible. React Router's `navigate` treats the value as a path segment, not a URL. No redirect risk.

## Verdict

No security issues. All data flows through existing authenticated infrastructure. Markdown rendering is safe. Thread ID generation cannot produce injection payloads.
