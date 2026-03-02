---
session: 2
timestamp: 2026-03-02T19:00:00Z
orientation:
  current: "Thread primitive is well-defined: FeedbackThread (body + versioned synthesis + comments) as a distinct lightweight primitive from PonderEntry. V1 scope locked (create + comment + show). V2 scope split off (synthesis + promote-to-ponder). Two open questions: navigation placement and title-only quick capture."
  next: "Jordan confirms: (a) Threads nav — standalone or embedded in Ponder? (b) Title-only capture allowed? Then spec uat-feedback-threads as a new feature."
  commit: "Commit signal met: Thread primitive is defined, data model agreed, V1/V2 scope split decided. Ready to spec and implement."
---

**jordan · Owner**
i think that idea is fine but i need the actual feedback experience to improve, when i start a feedback it should become a thread that has a core element that can update, then when other people comment it synthesizes it and adds it to the thread and then you can turn the thread into a ponder experience

---

**Facilitator**

Session 1 converged on: contextual Comments attached to ponder slugs, flat list, `resolved: bool`. That model is still valid — but Jordan just described something more structured. Three new concepts:

1. **Core element** — a thread isn't a list of comments; it has a *living center* that evolves
2. **Synthesis** — contributions fold into the core element automatically (implies agent involvement)
3. **Promote to ponder** — a thread can graduate to a full ideation workspace

Let me interrogate each concept before we redesign.

**"Core element that can update"** — is this:
- A Markdown document (like a mini-spec that grows)?
- A title + summary paragraph?
- The original message (like a GitHub issue body)?

**"Synthesizes it and adds to the thread"** — agent synthesizes comments → updates core element? Or creates a synthesis comment? Who triggers it?

**"Turn into a ponder experience"** — promotes the thread to a ponder entry. The ponder → comments direction from Session 1 is *reversed* here. Both can coexist.

---

**Nadia Osei · Rich Document Builder**

This is exactly the GitHub issue model, and I mean that as a compliment not a criticism.

GitHub issues have: a **body** (living, editable, the core element), **comments** (append-only), and timeline events. The body is structurally different from comments. It gets updated by the author. Comments react to it.

Jordan is describing that but with two additions:
- **Synthesis**: the body should incorporate comment insights automatically (an agent rewrites it using comment content)
- **Graduation**: the thread can become a ponder entry

The data model from Session 1 needs one structural change. Instead of a flat comment list, we have:

```
FeedbackThread {
  id: T1
  title: "My feedback idea"
  body: "Current living summary — updated by synthesis"
  author: "jordan"
  created_at: ...
  promoted_to: null | "<ponder-slug>"
  comments: [{ id, author, body, created_at, incorporated }]
}
```

The `body` is the core element. Comments contribute to it via synthesis. The thread can be promoted.

This is not Comments-on-a-ponder. This is **a standalone thread primitive** that *precedes* ponder.

⚑ **Decided:** A `FeedbackThread` is a first-class entity with a `body` (core element) separate from its `comments`. Not the same as Comments-on-a-ponder from Session 1.

---

**Felix Wagner · Tooling Architect**

Nadia, I agree on the structure — but where do threads live? Session 1 anchored comments to an existing slug (`.sdlc/roadmap/<slug>/comments.yaml`). Jordan's new model suggests threads can exist *without* a ponder entry — they're pre-ponder.

Storage options:
- `A` — `.sdlc/threads/<slug>/` — threads as top-level citizens, independent of ponder
- `B` — `.sdlc/roadmap/<slug>/threads/<thread-id>.yaml` — threads attached to a ponder slug
- `C` — Feedback threads replace the current `.sdlc/feedback.yaml` global queue

The "turn into a ponder experience" phrase demands option A — threads are independent and can be promoted. Otherwise "turn into ponder" would be circular (a ponder-attached thread becoming a ponder? same thing).

**Option A.** Threads are independent. They can optionally reference a `ponder_slug` if attached, and set `promoted_to_slug` when graduated.

? **Open:** Can a thread be attached to an existing ponder as a comment mechanism, *and* also be promoted to its own new ponder? That's two different use cases that look like one feature.

---

**Dana Cho · Product Skeptic**

I want to slow down on synthesis.

"When other people comment it synthesizes it and adds it to the thread" — this is automatic agent synthesis. That's a significant architectural commitment.

**What does synthesis failure look like?** If someone comments "I disagree with all of this", does the agent synthesize that into the core element as "there is disagreement"? Does it overwrite the author's original intent?

I've seen this go wrong at Linear. The synthesis felt authoritative when it was actually wrong. Users stopped trusting the body because they didn't know what was "them" vs "the agent's summary."

Two safer synthesis models:
1. **Agent synthesizes into a separate section** — body has `[Original]` + `[Updated]` sections. Original never overwritten.
2. **Synthesis creates a new comment** — agent posts "Here's what's emerged:" as a comment, not a body update. Author can manually promote it.

? **Open:** Is automatic body mutation acceptable, or should synthesis always be additive (new content appended, never erased)?

---

**Aria Chen · Agent Ergonomics**

Dana's concern is real but solvable with versioning + clear attribution.

The core element should be a **versioned document**, not a mutable string. Every synthesis produces a new version. The UI shows:
- Current body (latest version)
- Version history (who updated it, what triggered the update)
- "Last synthesized from N comments at [time]"

This solves the trust problem: the human can always see what the body was before synthesis ran, and what comments triggered the change. If the synthesis is wrong, they revert (one click).

From an agent perspective: synthesis is a run triggered by new comments. The agent reads the current body + all unincorporated comments, produces a revised body, writes it as a new version, and marks those comments `incorporated: true`.

⚑ **Decided:** The core element (`body`) is **versioned**. Every synthesis creates a new version file. Comments get `incorporated: bool` to track what's been absorbed.

---

**Nadia Osei**

Aria's versioning model is right but let me simplify the data shape so it doesn't become a database:

```yaml
# .sdlc/threads/<slug>/manifest.yaml
id: T1
title: "My feedback thread"
author: jordan
created_at: 2026-03-02T19:00:00Z
promoted_to: null
body_version: 3

# .sdlc/threads/<slug>/body-v1.md  — original
# .sdlc/threads/<slug>/body-v2.md  — first synthesis
# .sdlc/threads/<slug>/body-v3.md  — second synthesis

# .sdlc/threads/<slug>/comments.yaml
- id: C1
  author: jordan
  body: "Initial seed"
  created_at: ...
  incorporated: true
- id: C2
  author: teammate@
  body: "What about the edge case?"
  created_at: ...
  incorporated: false
```

Versioned body as flat `.md` files. Comments with `incorporated: bool`. Simple manifest.

CLI: `sdlc thread create <title>`, `sdlc thread comment <slug>`, `sdlc thread synthesize <slug>`, `sdlc thread promote <slug>`.

---

**Felix Wagner**

I want to resolve the naming tension. In the codebase we have `ponder`. Now we're proposing `thread`. Are these:
- Two separate concepts in the UI? (Threads tab + Ponder tab?)
- Or is "thread" the UX word for what's technically a `ponder` entry at lighter weight?

The "turn into a ponder experience" phrase suggests threads are *lighter* than ponders — they're the entry ramp. Ponders have sessions, teams, scrapbooks, phase tracking. Threads have a body, comments, synthesis. Different weight, different purpose.

⚑ **Decided:** `thread` is a distinct primitive from `ponder`. Threads are lightweight (body + comments + synthesis). Ponders are heavyweight (sessions + team + scrapbook + phases). Promotion lifts a thread into a ponder.

? **Open:** What carries over during promotion? Title → ponder title, body-v(latest) → first scrapbook artifact, comments → initial session log?

---

**Dan Reeves (via proxy) · Systems Minimalist**

Scope risk flag.

We're now talking about: threads as a new top-level primitive, versioned body files, synthesis agent runs, `incorporated` tracking on comments, a promote-to-ponder workflow, and migration of FeedbackPage to threads. That's four separate buildable features.

Minimum thread that proves the concept:
1. `sdlc thread create <title> --body "..."` → creates `.sdlc/threads/<slug>/`
2. `sdlc thread comment add <slug> --author <n> "body"` → appends to comments.yaml
3. `sdlc thread show <slug>` → renders body + comments

Synthesis and promotion are **two separate follow-on features**, not V1.

⚑ **Decided:** Thread V1 = create + comment + show. Synthesis and promotion are V2, scoped separately.

---

**Dana Cho**

Dan's right on scope. But the data model in V1 must *accommodate* synthesis and promotion — we shouldn't have to reshape the data layer when V2 arrives.

Add `promoted_to: null` and `body_version: 1` to the manifest even in V1. They're just unused. Add `incorporated: false` to comments even in V1. The V2 agent just starts using them.

One of the rare cases where a forward-compatible field costs nothing and saves a painful migration.

⚑ **Decided:** V1 data model includes `promoted_to`, `body_version`, and comment `incorporated` fields — stubbed to defaults, exercised in V2.

---

**Facilitator — Synthesis**

**What changed from Session 1:**

Session 1's "Comments on ponder entries" is *still valid* — it describes the annotation layer on an existing idea. What Jordan described in this seed is a *pre-ponder primitive*: a lightweight feedback thread that can graduate.

**Updated model: Two surfaces, one funnel**

```
FeedbackThread (new)                    PonderEntry (existing)
─────────────────────────               ──────────────────────
Entry point: create a thread            Entry point: /sdlc-ponder-commit
Lightweight: body + comments            Heavyweight: sessions + team + scrapbook
Synthesis: agent rewrites body (V2)     Synthesis: agent writes session logs
Graduation: promote → PonderEntry (V2)  No graduation (it's the destination)
Storage: .sdlc/threads/<slug>/          Storage: .sdlc/roadmap/<slug>/
```

The FeedbackPage becomes: "Create a new thread" — not a global capture queue.

**V1 scope for FeedbackThread:**
```
CLI:
  sdlc thread create <title> [--body "initial text"]
  sdlc thread comment add <slug> [--author <n>] "body"
  sdlc thread comment list <slug>
  sdlc thread show <slug>
  sdlc thread list

Storage:
  .sdlc/threads/<slug>/manifest.yaml
  .sdlc/threads/<slug>/body-v1.md
  .sdlc/threads/<slug>/comments.yaml

REST:
  POST   /api/threads
  GET    /api/threads
  GET    /api/threads/:slug
  POST   /api/threads/:slug/comments
  GET    /api/threads/:slug/comments

UI:
  FeedbackPage → becomes "Create Thread" or redirects to Threads list
  Thread detail: title + body (core element) + comment list + compose box
  New nav section (placement: standalone vs embedded in Ponder — TBD)
```

**V2 scope (separate features):**
```
- sdlc thread synthesize <slug>  — agent synthesis run, creates body-vN.md
- sdlc thread promote <slug>     — creates ponder entry from thread
- Agent skill instructions: check open threads before ending runs
```

? **Open:** Does the UI have a "Threads" nav item alongside Ponder, or are threads embedded *within* the Ponder workspace?

? **Open:** Can a thread be created without a body (just a title) as a quick-capture mechanism?

? **Open (V2):** What carries over during promotion — title, latest body as scrapbook artifact, comments as initial session log?

**Commit signal met.** Thread primitive is defined, data model agreed, V1/V2 scope split decided. Status → converging.
