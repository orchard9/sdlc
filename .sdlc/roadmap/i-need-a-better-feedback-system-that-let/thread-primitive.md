# Thread Primitive — Data Model

## Structure

```yaml
# .sdlc/threads/<slug>/manifest.yaml
id: T1
title: "My feedback thread"
author: jordan
created_at: 2026-03-02T19:00:00Z
promoted_to: null       # null | ponder-slug (V2)
body_version: 1         # incremented on each synthesis run (V2)
```

```markdown
<!-- .sdlc/threads/<slug>/body-v1.md — the core element, versioned -->
Initial description of the idea.
```

```yaml
# .sdlc/threads/<slug>/comments.yaml
- id: C1
  author: jordan
  body: "Initial seed"
  created_at: 2026-03-02T19:01:00Z
  incorporated: true    # V2: synthesis has absorbed this comment
- id: C2
  author: teammate@example.com
  body: "What about the edge case?"
  created_at: 2026-03-02T19:05:00Z
  incorporated: false
```

## Decisions

- ⚑ FeedbackThread is a **distinct primitive** from PonderEntry. Lighter weight.
- ⚑ The **core element (`body`)** is versioned — body-v1.md, body-v2.md, etc. Each synthesis creates a new version.
- ⚑ Comments have `incorporated: bool` — tracks what synthesis has absorbed.
- ⚑ Manifest includes `promoted_to` and `body_version` in V1 (stubbed) for V2 forward-compat.
- ⚑ Storage: `.sdlc/threads/<slug>/` — top-level, not nested under roadmap or features.

## Two surfaces, one funnel

```
FeedbackThread (new)                    PonderEntry (existing)
─────────────────────────               ──────────────────────
Entry point: create a thread            Entry point: /sdlc-ponder-commit
Lightweight: body + comments            Heavyweight: sessions + team + scrapbook
Synthesis: agent rewrites body (V2)     Synthesis: agent writes session logs
Graduation: promote → PonderEntry (V2)  No graduation (destination)
Storage: .sdlc/threads/<slug>/          Storage: .sdlc/roadmap/<slug>/
```

## V1 Scope

**CLI:**
- `sdlc thread create <title> [--body "..."]`
- `sdlc thread comment add <slug> [--author <name>] "body"`
- `sdlc thread comment list <slug>`
- `sdlc thread show <slug>`

**REST:**
- `POST   /api/threads` — create thread
- `GET    /api/threads` — list all
- `GET    /api/threads/:slug` — show thread + comments
- `POST   /api/threads/:slug/comments` — add comment
- `GET    /api/threads/:slug/comments` — list comments

**UI:**
- FeedbackPage becomes: "Create a new thread"
- Thread detail page: title + body (core element) + comment thread
- New nav section (placement TBD — standalone vs embedded in Ponder)

## V2 Scope (separate features)

- `sdlc thread synthesize <slug>` — agent reads body + unincorporated comments, rewrites body as v(N+1), marks comments `incorporated: true`
- `sdlc thread promote <slug>` — creates ponder entry; body-vN.md becomes first scrapbook artifact; comments become initial session log
- Agent skill instructions: check open threads before ending a run

## Open Questions

- ? Navigation: "Threads" as a standalone nav item, or embedded within the Ponder workspace?
- ? Can a thread be created with no body (title-only quick capture)?
- ? Does synthesis need human approval before the body version commits, or is it auto-committed?