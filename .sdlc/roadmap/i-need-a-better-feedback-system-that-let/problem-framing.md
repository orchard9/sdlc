# Problem Framing — Why the Current System Fails

## The core mismatch

The current feedback system is a **capture queue** (global, anonymous, temporary).
Jordan needs a **collaboration layer** (contextual, identified, persistent, living).

| Dimension        | Current (feedback.yaml)      | Needed (comments)              |
|------------------|------------------------------|--------------------------------|
| Attachment       | Global — no slug             | Per ponder/feature slug        |
| Identity         | None — anonymous             | `author` field (human + agent)|
| Persistence      | Cleared on submit            | Append-only, never deleted     |
| Lifecycle        | Pending → (deleted)          | Open → Resolved                |
| Agent visibility | Only after destructive submit | Always readable, per-slug      |
| UI surface       | Standalone FeedbackPage      | Embedded in PonderPage         |

## What "living idea" means

A ponder entry is a living idea. It accumulates:
- Scrapbook artifacts (captured via `sdlc ponder capture`)
- Session logs (agent thinking over time)
- **Comments** (human and agent reactions to the idea as it evolves)

The comment thread IS the collaboration surface on the living idea.
It doesn't replace the artifact — it annotates it.

## What "multiple people" means (from session)

Three possible interpretations, all supported by the same model:
1. **Human + Agent** — primary case; agent resolves comments it addresses
2. **Multiple humans** — team members accessing via tunnel auth
3. **Both** — unified by `author` field (`jordan`, `agent:advisor`, `teammate@`)

No special collaboration infrastructure needed — the `author` field + tunnel auth + git are enough.
