# QA Results: Wire up Thread Detail Actions

**Date:** 2026-03-03  
**Method:** Automated cargo tests + curl against temp API server + TypeScript type-check

## TC-7 — Backend routes and cargo tests

```
SDLC_NO_NPM=1 cargo test -p sdlc-core -p sdlc-server
test result: ok. 45 passed; 0 failed
```

```
cargo clippy --all -- -D warnings
Finished `dev` profile — no warnings
```

**PASS**

## TC-8 — Frontend type-check

```
cd frontend && npx tsc --noEmit
(no output — zero errors)
```

**PASS**

## TC-1 — Create thread, verify status=open

```json
{"status": "open", "promoted_to": null, "slug": "20260303-general"}
```

**PASS** — `status` field present, defaults to `"open"`, `promoted_to` is null.

## TC-2 — GET thread returns status from storage

GET `/api/threads/20260303-general` → `"status": "open"`, `"promoted_to": null`

**PASS**

## TC-3 — PATCH /api/threads/:id sets status=synthesized

```
PATCH /api/threads/20260303-general {"status":"synthesized"}
→ {"status": "synthesized", "promoted_to": null}
```

**PASS**

## TC-4 — Status persists after PATCH (reload round-trip)

GET `/api/threads/20260303-general` after PATCH → `"status": "synthesized"`

**PASS**

## TC-5 + TC-6 — POST /api/threads/:id/promote creates ponder entry

```
POST /api/threads/20260303-general-2/promote
→ {"ponder_slug": "thread-to-promote", "thread_id": "20260303-general-2"}
```

GET `/api/roadmap/thread-to-promote` → `{"slug": "thread-to-promote", "title": "Thread to promote"}`

**PASS**

## TC-7 (promote status) — GET thread after promote

```
GET /api/threads/20260303-general-2
→ {"status": "promoted", "promoted_to": "thread-to-promote"}
```

**PASS**

## TC-1 (delete) — DELETE /api/threads/:id returns deleted:true

```
DELETE /api/threads/20260303-general → {"deleted": true}
```

**PASS**

## TC-2 (delete 404) — GET deleted thread returns 404

```
GET /api/threads/20260303-general → HTTP 404
```

**PASS**

## TC-11 — DELETE non-existent thread returns 404

```
DELETE /api/threads/does-not-exist → HTTP 404
```

**PASS**

## TC-9 — Existing comment flow unaffected

```
POST /api/threads/:id/comments {"author":"jordan","body":"Hello thread"}
→ {"author": "jordan", "body": "Hello thread", "incorporated": false}
```

**PASS**

## UI verification (code review)

ThreadDetailPane.tsx reviewed directly:
- Delete button (trash icon) present in header ✓
- Inline confirm state ("Delete? / Cancel") ✓  
- Synthesize button enabled when `status === 'open'`, disabled otherwise ✓
- Promote to Ponder enabled when `status !== 'promoted'`, disabled otherwise ✓
- All three show `<Loader2>` spinner during async ✓
- All three show inline error message on failure ✓
- StatusBadge handles all three statuses (was already correct) ✓

## Summary

**All QA cases: PASS**

| TC | Description | Result |
|----|-------------|--------|
| TC-7 | cargo test + clippy | PASS |
| TC-8 | tsc --noEmit | PASS |
| TC-1 | Create thread status=open | PASS |
| TC-2 | GET thread status persists | PASS |
| TC-3 | PATCH synthesize | PASS |
| TC-4 | Status round-trips | PASS |
| TC-6 | Promote creates ponder entry | PASS |
| TC-7b | Thread status after promote | PASS |
| TC-1b | Delete returns deleted:true | PASS |
| TC-2b | GET deleted → 404 | PASS |
| TC-11 | DELETE nonexistent → 404 | PASS |
| TC-9 | Comment flow unaffected | PASS |
