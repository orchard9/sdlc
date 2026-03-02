# Design Decisions — Contextual Comment System

## What's broken with feedback.yaml

- **Global, not contextual** — notes have no attachment to a specific ponder or feature slug
- **Anonymous** — no author field; impossible to know who said what
- **Destructive submit** — "Submit to Ponder" clears the queue after bundling (one-shot, non-reversible)
- **Invisible to agents** — agents can't read pending notes without consuming them
- **No resolution lifecycle** — notes are either pending or deleted; no "addressed" state

## Core Primitive: Comment

```yaml
# .sdlc/roadmap/<slug>/comments.yaml
- id: C1
  author: jordan               # or "agent:advisor" for agent attribution
  body: "Some feedback text"
  created_at: 2026-03-02T18:00:00Z
  updated_at: 2026-03-02T18:00:00Z
  resolved: false
```

### Decisions (resolved)

- ⚑ Primitive is **Comment**, not FeedbackNote. Identity (author) is non-negotiable.
- ⚑ Author supports **`agent:<name>` prefix** — agents distinguishable from humans in thread.
- ⚑ **Flat list only** — no `parent_id`, no nested threading in V1. Single resolved/open lifecycle.
- ⚑ **`resolved: bool`** is the only state field. Comments don't delete — they resolve.
- ⚑ **Primary UI** is a comment thread at the bottom of PonderPage — not the standalone FeedbackPage.
- ⚑ FeedbackPage stays as **global capture fallback** — not removed, just no longer the primary surface.

## V1 Scope (ponder-only)

**In:**
- Comment CRUD on ponder slugs
- Storage: `.sdlc/roadmap/<slug>/comments.yaml`
- CLI: `sdlc ponder comment add|list|resolve <slug>`
- REST: `POST|GET /api/ponder/:slug/comments`, `PATCH /api/ponder/:slug/comments/:id`
- UI: comment thread in PonderPage (compose box + list with resolve action)
- Agent skills updated to check/resolve open comments before ending a run

**Out of V1:**
- Feature-level comments (generalize after ponder works)
- Threading / parent_id
- Real-time SSE for new comments
- Migration of existing feedback.yaml items

## Open Questions

- ? Should `--author` default to `git config user.name` when omitted?
- ? Is SSE notification for new comments needed at launch, or can UI poll on page load?
