---
session: 1
timestamp: 2026-03-02T18:30:00Z
orientation:
  current: "Identified core gap: feedback.yaml is a global capture queue; Jordan needs a collaboration layer — contextual, identified, persistent, threaded on living artifacts. Three candidate models surfaced: comments on ponder entries, enrichment protocol on any slug, or a dedicated comment thread primitive. Leaning toward comments as a first-class primitive on ponder slugs, then extend to features."
  next: "Prototype the data model: Comment { id, slug, author, body, created_at, resolved, parent_id? }. Start with ponder. Decide whether to unify with or replace existing feedback.yaml queue."
  commit: "Commit when data model is agreed and minimum interaction surface (post/list/resolve) is defined for a single artifact type (ponder entry)."
---

**jordan · Owner**
i need a better feedback system that lets multiple people collaborate on a living idea through an artifact

---

**Facilitator**
Fresh ponder entry — no prior context. Before we explore solutions, let's interrogate what's actually broken. I've read the current system:

- `feedback.yaml` — global note queue, no author field, no artifact attachment
- "Submit to Ponder" — bundles all notes into a new ponder entry and **clears the queue**
- Enrichments exist (agent-only) but no UI surface for them

Jordan, "collaborate on a living idea through an artifact" is doing a lot of work. Let me surface the real problems before we build anything.

Three possible interpretations:
1. **People need to leave notes on a specific ponder entry** (not a global queue)
2. **People need to see and respond to each other's notes** (threading/identity)
3. **The idea should evolve in response to feedback** (the artifact itself updates, like a wiki)

Which of these is closest to the friction you're feeling?

---

**Nadia Osei · Rich Document Builder**
I've been here before. Three times actually. At Coda, at Notion, at Linear.

The pattern that fails every time: you build a "feedback" system without deciding *what the feedback is for*. Is it for the human collaborator reading the ponder entry? For the agent that will act on it? For the author who needs to know what changed?

My honest take: **the current FeedbackPage is almost the right idea, but the wrong attachment point**. Right now it's floating — a global queue with no home. The fix isn't to rebuild the system, it's to anchor each note to a slug.

The minimum viable change: add `slug` and `author` to `FeedbackNote`. Store feedback per-slug at `.sdlc/roadmap/<slug>/feedback.yaml` instead of `.sdlc/feedback.yaml`. Suddenly you have contextual, identity-aware feedback on a specific artifact, and the UI can show it in context.

? **Open:** Does this also need to work on feature artifacts (`.sdlc/features/<slug>/`) or only ponder entries?

---

**Felix Wagner · Tooling Architect**
Nadia's right about the slug anchor, but I want to challenge the model deeper.

The word "collaborate" implies that multiple people are *reacting to each other's notes*, not just the artifact. That's a fundamentally different data model:

- **Feedback queue**: people react to an idea → notes accumulate → agent processes them
- **Comment thread**: people react to *each other* → threading matters → the thread is itself a record

The current `FeedbackNote` with `Enrichment` is almost a thread already — `Enrichment` is basically a reply. We just haven't named it that or given it identity.

My proposal: rename the core primitive. `FeedbackNote` → `Comment`. `Enrichment` → `Reply`. Add `author` to both. Store per-slug. The REST API stays almost the same: POST, GET, PATCH, DELETE — just contextualized by slug.

⚑  **Decided:** The primitive should be `Comment`, not `FeedbackNote`. Identity (author field) is non-negotiable.

? **Open:** Should the existing `feedback.yaml` global queue be deprecated entirely, or kept as a quick-capture surface that drains into contextual comments?

---

**Dana Cho · Product Skeptic**
Hold on. Who are the "multiple people" here?

If this is a solo developer using sdlc to manage their own project, "multiple people" is them plus the AI agent. That's a very different collaboration model than "a team of 5 engineers leaving async comments on a ponder entry."

Jordan said "multiple people" — but is this:
a) **Human + Agent** (agent enriches notes, human reads them)
b) **Multiple humans** (team members leaving async notes via the tunnel auth surface)
c) **Both**

This matters because scenario (a) is already half-built — the `Enrichment` type exists for agent-side notes. Scenario (b) requires identity and a way for remote collaborators to post (the tunnel auth).

I'm going to push back on the scope here: **don't build scenario (b) collaboration until you've validated that tunnel-auth users actually need to leave notes**. The simpler question is: does the current feedback system fail you even as a solo user with an agent? If yes, fix that first.

? **Open:** Is the core friction that notes are global (no artifact context), or that they're anonymous, or that you can't see what the agent has responded to?

---

**Aria Chen · Agent Ergonomics**
I'll answer Dana's question from the agent side.

The current system fails agents in one specific way: **feedback notes are invisible to agents until they're "submitted to ponder"**. That submit action is destructive — it clears the queue. Agents can't read pending notes, can't enrich specific notes, and can't know which notes have been acted on vs ignored.

From an agent perspective, what's needed is:
1. **Readability without submission** — agents should be able to read feedback on a ponder entry mid-session without consuming/clearing the queue
2. **Enrichment attribution** — when an agent enriches a note, the UI should show "Agent responded" — this closes the loop for the human
3. **Resolution state** — notes should have `open` / `resolved` states. Agents can mark notes as addressed. Humans can see what's been handled.

The storage model Nadia proposed (per-slug `feedback.yaml` or `comments.yaml`) solves #1 immediately. Resolution state is a one-field addition.

⚑  **Decided:** Comments need a `resolved: bool` field. This is the minimum viable lifecycle. Agents write `resolved: true` after addressing a note.

---

**Nadia Osei**
Aria's point about the destructive submit is the most important thing said so far. Let me be precise about what should replace it:

The "Submit to Ponder" button on the FeedbackPage is wrong in two ways:
1. It's destructive (clears notes)
2. It's disconnected from context (creates a *new* ponder entry from global notes, rather than adding to an existing one)

What the user actually wants: **leave a note on this specific ponder entry and have it persisted alongside the scrapbook**. The UI affordance should be a comment box at the bottom of the PonderPage — not a separate FeedbackPage at all.

The FeedbackPage becomes vestigial once we move to contextual comments. Keep it as a quick-capture surface for thoughts without a home yet, but the main collaboration happens in context.

⚑  **Decided:** Primary collaboration surface is a comment thread on the PonderPage (and FeaturePage), not the standalone FeedbackPage. FeedbackPage becomes a "capture without context" fallback only.

---

**Felix Wagner**
Let me nail down the data model so we have something concrete:

```yaml
# .sdlc/roadmap/<slug>/comments.yaml
# or .sdlc/features/<slug>/comments.yaml
- id: C1
  author: jordan
  body: "Should we also handle the case where the spec is rejected twice?"
  created_at: 2026-03-02T18:00:00Z
  updated_at: 2026-03-02T18:00:00Z
  resolved: false
  replies:
    - id: R1
      author: agent:advisor
      body: "Yes — the state machine handles this: spec=Rejected → create_spec loops"
      created_at: 2026-03-02T18:01:00Z
```

Two lifecycle exits for a comment:
1. `resolved: true` — addressed, archived from the active view
2. Promoted to a feature task via `sdlc task add` (rare, for big findings)

Comments don't delete. They resolve. Resolved comments collapse but are retrievable.

? **Open:** Does `replies` as a nested array get too complex? Alternative: flat comments with `parent_id: "C1"` for threading. Flat is simpler for the data layer and for CLI.

---

**Dan Reeves (via proxy) · Systems Minimalist**
I'll offer the skeptical take: `parent_id` flat threading is one extra field, infinite nesting is free, and it's trivially queryable. Nested arrays require recursive rendering logic. Go flat.

But also: do we need threading at all for V1? A comment with `resolved: bool` is probably enough. The agent responds by resolving + optionally adding a new top-level comment with its response. Full threading is week 6 scope.

⚑  **Decided:** V1 = flat comments, no `parent_id`. `resolved: bool` is the only state field. Threading is a future enhancement.

---

**Dana Cho**
Good. Now let me scope the actual feature:

**Must-have for V1:**
- `Comment { id, author, body, created_at, resolved }` on ponder slugs
- Storage: `.sdlc/roadmap/<slug>/comments.yaml`
- CLI: `sdlc ponder comment add <slug> --author <name> "body"` (or `--author` inferred from git config)
- REST: `POST /api/ponder/:slug/comments`, `GET /api/ponder/:slug/comments`, `PATCH /api/ponder/:slug/comments/:id` (resolve)
- UI: comment thread at bottom of PonderPage with compose box

**Out of V1:**
- Feature-level comments (do ponder first, then generalize)
- Threading / parent_id
- Reactions / emoji
- Real-time SSE for comment updates (though easy to add later)
- Migrating existing feedback.yaml items

**Migration of FeedbackPage:**
- Keep FeedbackPage as-is for now (don't break existing users)
- Add a "Move to [Ponder]" action on individual notes as a forward-path
- Deprecate "Submit to Ponder" later once contextual comments are the norm

---

**Aria Chen**
One more thing for the agent interface: `author` should accept `agent:<slug>` as a valid prefix so agents are distinguishable from humans in the thread. The UI can render these differently — a small chip or icon rather than a name badge.

Agents will use this in `sdlc-run` when they process feedback:
```bash
sdlc ponder comment add <slug> --author "agent:advisor" "Addressed in spec revision — see design.md §3"
sdlc ponder comment resolve <slug> C3
```

The `sdlc-run` and `sdlc-next` skills should include instructions: "Before ending a run, check for open comments on the active ponder entry and address or resolve them."

⚑  **Decided:** `author` field supports `agent:<name>` prefix for agent attribution. UI renders agent comments with a distinguishing visual treatment.

---

**Facilitator — Synthesis**

**What we're building:** A contextual comment thread on ponder entries (and later features), replacing the global "feedback queue" pattern with identified, persistent, resolvable comments.

**Core model:**
```
Comment {
  id: C1, C2...
  author: "jordan" | "agent:advisor" | "teammate@example.com"
  body: String
  created_at: DateTime
  updated_at: DateTime
  resolved: bool
}
```

**Storage:** `.sdlc/roadmap/<slug>/comments.yaml`
**CLI:** `sdlc ponder comment add|list|resolve <slug>`
**REST:** `POST|GET /api/ponder/:slug/comments`, `PATCH /api/ponder/:slug/comments/:id`
**UI:** Comment thread in PonderPage, below scrapbook artifacts

**What changes for FeedbackPage:** Keep as global capture fallback. Don't break it. Add forward path ("add to ponder") on individual notes. Long-term, deprecate.

**What changes for agents:** `sdlc-run` / `sdlc-next` skills instruct agents to check and resolve open comments before completing a run.

? **Open:** Should comment identity be inferred from git config (`git config user.name`) when `--author` is omitted? This makes the CLI friction-free for the primary author.

? **Open:** Is SSE notification for new comments needed at launch, or can the UI poll on page load?

**Commit signal:** Model is agreed. Minimum surface defined. Ready to spec once Jordan confirms the V1 scope (ponder-only, flat, no threading).
