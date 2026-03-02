---
session: 1
timestamp: 2026-03-01T00:00:00Z
orientation:
  current: "Two distinct improvements scoped: inline edit and MCP-driven note enrichment. Both have clear implementation paths."
  next: "Write spec — capture data model changes (update fn + PATCH route + optional enrichments field), UX for inline edit, and enrichment flow (agent call on a note, append findings)."
  commit: "Spec written and design aligned. Proceed to feature backlog."
---

**jordan · Owner**
feedback improvements

we want to improve the feedback - should be able to edit/delete feedback, should be able to run an mcp call to add more information to feedback

---

## Session 1 — Scoping the two improvements

**Facilitator** opening the session with the brief. Three thought partners recruited: **Ben Hartley** (developer productivity UX), **Felix Wagner** (data model/lifecycle), and **Dan Reeves** (systems minimalist).

---

### What we're actually working with

Before diving in: the current feedback system is a simple append-only note queue. Here's what exists:

- `feedback.rs` — `add`, `list`, `delete`, `clear`, `to_markdown`. No `update`.
- Server routes — `GET /api/feedback`, `POST /api/feedback`, `DELETE /api/feedback/:id`, `POST /api/feedback/to-ponder`
- `FeedbackPage.tsx` — compose textarea + save, note cards with double-click-to-confirm delete, submit-all-to-ponder button
- `/__sdlc/feedback` — public tunnel alias (external agents can add notes)

**Delete already exists.** The stated requirement conflates two things: (a) delete is there, (b) **edit is missing**. So the first improvement is edit. The second is MCP enrichment.

---

### Improvement 1: Edit feedback notes

**Ben Hartley** weighs in first.

> "Inline edit is the right call here — don't open a modal. The note card already has the text in a `<pre>`. When the user clicks into the content area, swap to a textarea in-place, same styling. ⌘+Enter saves, Escape cancels. Delete stays on the card in non-edit mode. This is the GitHub inline comment edit pattern."
>
> "The trap to avoid: don't add an edit button as a separate affordance. Edit-on-click is discoverable enough for this audience (developers). A pencil icon in the top-right adds visual noise to every card."

**Dan Reeves** challenges the premise.

> "Wait — delete already exists. Edit is basically 'delete and re-create with corrected text.' Why do we need a mutation primitive? Is there actual evidence that users are making typos in quick-capture notes and then wanting to fix them rather than just deleting?"
>
> "Counter-argument from myself: the note has an ID (F1, F2, ...) that you might want to preserve for any downstream reference. If you delete-and-recreate, you get a new ID. That's a weak argument but it's the strongest one for edit."

**Felix Wagner** on the data model side.

> "The ID preservation argument is real but thin. The stronger case: if you're building toward enrichments (second improvement), you need edit to be a first-class mutation because enrichments will attach to a note ID. Delete-recreate breaks that link."
>
> "Data model for edit: trivially simple. `update(root, id, new_content)` in `feedback.rs` — load all, find by ID, replace content, atomic write. PATCH `/api/feedback/:id` with `{ content: string }` body. No new fields. Five minutes of Rust."

⚑  **Decided:** Edit is justified for ID preservation and forward-compatibility with enrichments. Inline edit UX (click-to-edit, not a button), ⌘+Enter to save, Escape to cancel.

---

### Improvement 2: MCP call to add more information

This is the more interesting requirement. Let's interrogate what "run an mcp call to add more information to feedback" actually means.

**Facilitator** proposes three interpretations:

**A.** Agent-driven enrichment: from the UI, trigger a Claude agent run that reads the codebase/project state and appends structured context to a note. E.g., note says "dashboard is slow" → agent reads telemetry/runs/advisory → appends: "See runs from Feb 28: 3 runs exceeded 30s. Likely cause: feature list SSE polling."

**B.** External tool injection: use a Playwright MCP call (or fetch) to pull in external information — e.g., link to a GitHub issue, doc page, error trace — and append it to the note.

**C.** MCP tool exposure: expose `POST /api/feedback` as an MCP tool descriptor so agents (in Claude Code or other contexts) can call it to add notes programmatically, possibly with richer metadata than the basic `content` string.

**Dan Reeves** cuts through.

> "C is already done — `/__sdlc/feedback` is exactly that. Don't reinvent it. Cross off C."
>
> "B and A are different at the invocation layer but similar at the data model layer. Both produce additional text that gets attached to an existing note. The question is: does the extra content live inside `content` (appended), or does it go into a new `enrichments: []` field on the note?"

**Ben Hartley** on UX implications.

> "If enrichments are appended into content, the note becomes a monolithic blob. The user loses the boundary between what they wrote and what the agent added. That boundary matters — it affects trust. You want to see 'here's what I wrote, here's what the agent found.'"
>
> "So: `enrichments: Vec<Enrichment>` where each has `source` (the tool/agent that added it), `content`, `added_at`. The card renders the original content, then below a subtle divider, enrichment blocks with a different visual treatment — muted background, maybe a small sparkle icon for agent-added content."

**Felix Wagner** on the data model decision.

> "Agreed on separate `enrichments` field. It also makes serialization clean — the base note stays stable, enrichments append. No rewriting the main content string."
>
> "For 'B — external URL/page pull': this is actually simpler than running a full agent. User pastes a URL into a text field on the note card, we fetch the page title + summary, append as an enrichment. No MCP protocol needed, just `POST /api/feedback/:id/enrich` with `{ url: string }` or `{ content: string, source: string }`."

**Dan Reeves** pushes back on enrichment complexity.

> "You're describing a new data sub-model (`enrichments` field + new route) for a feature that hasn't been validated. Start with the simpler path: `POST /api/feedback/:id/enrich` takes `{ content: string, source: string }` and appends to an `enrichments: Vec` on the note. That's it. No URL-fetch, no agent invocation inside the server — the agent runs externally (via `spawn_agent_run` or directly) and calls the enrich endpoint with pre-fetched content."
>
> "The MCP angle: the CLI or the server exposes an MCP tool `feedback_enrich` that takes `{ id, content, source }`. External agents (Claude Code running `/sdlc-run`) can call this while working on a feature — 'I found a relevant issue, let me enrich this feedback note.'"

⚑  **Decided:** `enrichments` as a separate field on `FeedbackNote`. Structure:
```rust
pub struct Enrichment {
    pub source: String,      // "agent:sdlc-run", "user:url", "mcp:fetch", etc.
    pub content: String,
    pub added_at: DateTime<Utc>,
}
```
Mutation: `POST /api/feedback/:id/enrich` with `{ content, source }`.

---

### The MCP tool angle

**Ben Hartley** surfaces the UX entry point question.

> "OK so the user wants to 'run an MCP call' — that phrase implies user agency, not agent-driven. They want to click something in the UI that triggers a research action. Not just 'agents can call this endpoint'; the user initiates it."
>
> "Proposal: each note card gets a small 'Research' action (⚡ icon). Clicking it opens a mini-prompt: 'What should the agent look for?' with a textarea. User types context, submits, agent runs (via `spawn_agent_run`), enrichment appears below the note content when done."

**Dan Reeves** on minimal footprint.

> "That's a full agent run per note. Cost and latency concerns for a quick-capture tool. Counter: start with manual enrichment. User clicks 'Add context', gets a textarea, types enrichment content themselves, saves it. The structured enrichment data model is in place. Agent-driven enrichment is an overlay on top — you flip the source from 'user:manual' to 'agent:run' but the storage layer is identical."

**Felix Wagner** on phasing.

> "Dan's right on sequencing. V1 of enrichment: manual `POST /api/feedback/:id/enrich` — user appends their own context (e.g., paste in a stack trace, link to a doc). V2: agent-driven enrichment via spawn_agent_run. The MCP tool (`feedback_enrich`) is V1-compatible — external agents can call it immediately."

⚑  **Decided:** Two-phase enrichment:
- **V1**: Manual enrichment — `POST /api/feedback/:id/enrich` with `{ content, source: "user" }`. UI: "Add context" button on note card. Text field opens below note, user types, saves. Source tagged as "user".
- **V2**: Agent-driven enrichment — UI prompt triggers `spawn_agent_run`, result POSTs back via enrich endpoint. Source tagged as "agent:research".

---

### Summary of scope

**Improvement 1 — Edit** (small, self-contained):
- `update(root, id, new_content) -> Result<FeedbackNote>` in `feedback.rs`
- `PATCH /api/feedback/:id` with `{ content: string }` body
- `api.updateFeedbackNote(id, content)` in client.ts
- Inline edit in `NoteCard`: click content area → textarea, ⌘+Enter saves, Escape cancels

**Improvement 2 — Enrichments** (medium, new sub-model):
- Add `Enrichment { source, content, added_at }` struct to `feedback.rs`
- Add `enrichments: Vec<Enrichment>` field to `FeedbackNote` (default empty)
- `enrich(root, id, source, content) -> Result<FeedbackNote>` function
- `POST /api/feedback/:id/enrich` with `{ content, source }` body
- `api.enrichFeedbackNote(id, content, source)` in client.ts
- Note card renders enrichment blocks below a divider, muted styling
- "Add context" button on hover expands an inline textarea for manual enrichment entry
- MCP tool descriptor: `feedback_enrich` registered in sdlc MCP server

**?  Open:** Should V2 agent-driven enrichment be scoped into the same feature or a follow-on? Lean toward follow-on — the data model + manual UI is a complete unit.

**?  Open:** Does `FeedbackNote` in `to_markdown()` need to render enrichments? Almost certainly yes — when bundling to ponder, all enrichment context should be included.

---

### WHERE WE ARE / NEXT MOVE / COMMIT SIGNAL

**WHERE WE ARE:** Two improvements fully scoped. Edit is trivial. Enrichments introduce a new sub-model (`enrichments: Vec<Enrichment>`) with V1 manual + V2 agent-driven phases.

**NEXT MOVE:** Write spec artifact (`feedback-improvements` ponder) capturing data model, API contract, UX decisions. Then break into two features: `feedback-edit` and `feedback-enrich`.

**COMMIT SIGNAL:** Spec written and reviewed. At that point, commit to two features.

---

---
session: 1
timestamp: 2026-03-01T00:00:00Z
orientation:
  current: "Two distinct improvements scoped: inline edit and MCP-driven note enrichment. Both have clear implementation paths."
  next: "Write spec — capture data model changes (update fn + PATCH route + optional enrichments field), UX for inline edit, and enrichment flow (agent call on a note, append findings)."
  commit: "Spec written and design aligned. Proceed to feature backlog."
---

**jordan · Owner**
feedback improvements

we want to improve the feedback - should be able to edit/delete feedback, should be able to run an mcp call to add more information to feedback

---

## Session 1 — Scoping the two improvements

**Facilitator** opening the session with the brief. Three thought partners recruited: **Ben Hartley** (developer productivity UX), **Felix Wagner** (data model/lifecycle), and **Dan Reeves** (systems minimalist).

---

### What we're actually working with

Before diving in: the current feedback system is a simple append-only note queue. Here's what exists:

- `feedback.rs` — `add`, `list`, `delete`, `clear`, `to_markdown`. No `update`.
- Server routes — `GET /api/feedback`, `POST /api/feedback`, `DELETE /api/feedback/:id`, `POST /api/feedback/to-ponder`
- `FeedbackPage.tsx` — compose textarea + save, note cards with double-click-to-confirm delete, submit-all-to-ponder button
- `/__sdlc/feedback` — public tunnel alias (external agents can add notes)

**Delete already exists.** The stated requirement conflates two things: (a) delete is there, (b) **edit is missing**. So the first improvement is edit. The second is MCP enrichment.

---

### Improvement 1: Edit feedback notes

**Ben Hartley** weighs in first.

> "Inline edit is the right call here — don't open a modal. The note card already has the text in a `<pre>`. When the user clicks into the content area, swap to a textarea in-place, same styling. ⌘+Enter saves, Escape cancels. Delete stays on the card in non-edit mode. This is the GitHub inline comment edit pattern."
>
> "The trap to avoid: don't add an edit button as a separate affordance. Edit-on-click is discoverable enough for this audience (developers). A pencil icon in the top-right adds visual noise to every card."

**Dan Reeves** challenges the premise.

> "Wait — delete already exists. Edit is basically 'delete and re-create with corrected text.' Why do we need a mutation primitive? Is there actual evidence that users are making typos in quick-capture notes and then wanting to fix them rather than just deleting?"
>
> "Counter-argument from myself: the note has an ID (F1, F2, ...) that you might want to preserve for any downstream reference. If you delete-and-recreate, you get a new ID. That's a weak argument but it's the strongest one for edit."

**Felix Wagner** on the data model side.

> "The ID preservation argument is real but thin. The stronger case: if you're building toward enrichments (second improvement), you need edit to be a first-class mutation because enrichments will attach to a note ID. Delete-recreate breaks that link."
>
> "Data model for edit: trivially simple. `update(root, id, new_content)` in `feedback.rs` — load all, find by ID, replace content, atomic write. PATCH `/api/feedback/:id` with `{ content: string }` body. No new fields. Five minutes of Rust."

⚑  **Decided:** Edit is justified for ID preservation and forward-compatibility with enrichments. Inline edit UX (click-to-edit, not a button), ⌘+Enter to save, Escape to cancel.

---

### Improvement 2: MCP call to add more information

This is the more interesting requirement. Let's interrogate what "run an mcp call to add more information to feedback" actually means.

**Facilitator** proposes three interpretations:

**A.** Agent-driven enrichment: from the UI, trigger a Claude agent run that reads the codebase/project state and appends structured context to a note. E.g., note says "dashboard is slow" → agent reads telemetry/runs/advisory → appends: "See runs from Feb 28: 3 runs exceeded 30s. Likely cause: feature list SSE polling."

**B.** External tool injection: use a Playwright MCP call (or fetch) to pull in external information — e.g., link to a GitHub issue, doc page, error trace — and append it to the note.

**C.** MCP tool exposure: expose `POST /api/feedback` as an MCP tool descriptor so agents (in Claude Code or other contexts) can call it to add notes programmatically, possibly with richer metadata than the basic `content` string.

**Dan Reeves** cuts through.

> "C is already done — `/__sdlc/feedback` is exactly that. Don't reinvent it. Cross off C."
>
> "B and A are different at the invocation layer but similar at the data model layer. Both produce additional text that gets attached to an existing note. The question is: does the extra content live inside `content` (appended), or does it go into a new `enrichments: []` field on the note?"

**Ben Hartley** on UX implications.

> "If enrichments are appended into content, the note becomes a monolithic blob. The user loses the boundary between what they wrote and what the agent added. That boundary matters — it affects trust. You want to see 'here's what I wrote, here's what the agent found.'"
>
> "So: `enrichments: Vec<Enrichment>` where each has `source` (the tool/agent that added it), `content`, `added_at`. The card renders the original content, then below a subtle divider, enrichment blocks with a different visual treatment — muted background, maybe a small sparkle icon for agent-added content."

**Felix Wagner** on the data model decision.

> "Agreed on separate `enrichments` field. It also makes serialization clean — the base note stays stable, enrichments append. No rewriting the main content string."
>
> "For 'B — external URL/page pull': this is actually simpler than running a full agent. User pastes a URL into a text field on the note card, we fetch the page title + summary, append as an enrichment. No MCP protocol needed, just `POST /api/feedback/:id/enrich` with `{ url: string }` or `{ content: string, source: string }`."

**Dan Reeves** pushes back on enrichment complexity.

> "You're describing a new data sub-model (`enrichments` field + new route) for a feature that hasn't been validated. Start with the simpler path: `POST /api/feedback/:id/enrich` takes `{ content: string, source: string }` and appends to an `enrichments: Vec` on the note. That's it. No URL-fetch, no agent invocation inside the server — the agent runs externally (via `spawn_agent_run` or directly) and calls the enrich endpoint with pre-fetched content."
>
> "The MCP angle: the CLI or the server exposes an MCP tool `feedback_enrich` that takes `{ id, content, source }`. External agents (Claude Code running `/sdlc-run`) can call this while working on a feature — 'I found a relevant issue, let me enrich this feedback note.'"

⚑  **Decided:** `enrichments` as a separate field on `FeedbackNote`. Structure:
```rust
pub struct Enrichment {
    pub source: String,      // "agent:sdlc-run", "user:url", "mcp:fetch", etc.
    pub content: String,
    pub added_at: DateTime<Utc>,
}
```
Mutation: `POST /api/feedback/:id/enrich` with `{ content, source }`.

---

### The MCP tool angle

**Ben Hartley** surfaces the UX entry point question.

> "OK so the user wants to 'run an MCP call' — that phrase implies user agency, not agent-driven. They want to click something in the UI that triggers a research action. Not just 'agents can call this endpoint'; the user initiates it."
>
> "Proposal: each note card gets a small 'Research' action (⚡ icon). Clicking it opens a mini-prompt: 'What should the agent look for?' with a textarea. User types context, submits, agent runs (via `spawn_agent_run`), enrichment appears below the note content when done."

**Dan Reeves** on minimal footprint.

> "That's a full agent run per note. Cost and latency concerns for a quick-capture tool. Counter: start with manual enrichment. User clicks 'Add context', gets a textarea, types enrichment content themselves, saves it. The structured enrichment data model is in place. Agent-driven enrichment is an overlay on top — you flip the source from 'user:manual' to 'agent:run' but the storage layer is identical."

**Felix Wagner** on phasing.

> "Dan's right on sequencing. V1 of enrichment: manual `POST /api/feedback/:id/enrich` — user appends their own context (e.g., paste in a stack trace, link to a doc). V2: agent-driven enrichment via spawn_agent_run. The MCP tool (`feedback_enrich`) is V1-compatible — external agents can call it immediately."

⚑  **Decided:** Two-phase enrichment:
- **V1**: Manual enrichment — `POST /api/feedback/:id/enrich` with `{ content, source: "user" }`. UI: "Add context" button on note card. Text field opens below note, user types, saves. Source tagged as "user".
- **V2**: Agent-driven enrichment — UI prompt triggers `spawn_agent_run`, result POSTs back via enrich endpoint. Source tagged as "agent:research".

---

### Summary of scope

**Improvement 1 — Edit** (small, self-contained):
- `update(root, id, new_content) -> Result<FeedbackNote>` in `feedback.rs`
- `PATCH /api/feedback/:id` with `{ content: string }` body
- `api.updateFeedbackNote(id, content)` in client.ts
- Inline edit in `NoteCard`: click content area → textarea, ⌘+Enter saves, Escape cancels

**Improvement 2 — Enrichments** (medium, new sub-model):
- Add `Enrichment { source, content, added_at }` struct to `feedback.rs`
- Add `enrichments: Vec<Enrichment>` field to `FeedbackNote` (default empty)
- `enrich(root, id, source, content) -> Result<FeedbackNote>` function
- `POST /api/feedback/:id/enrich` with `{ content, source }` body
- `api.enrichFeedbackNote(id, content, source)` in client.ts
- Note card renders enrichment blocks below a divider, muted styling
- "Add context" button on hover expands an inline textarea for manual enrichment entry
- MCP tool descriptor: `feedback_enrich` registered in sdlc MCP server

**?  Open:** Should V2 agent-driven enrichment be scoped into the same feature or a follow-on? Lean toward follow-on — the data model + manual UI is a complete unit.

**?  Open:** Does `FeedbackNote` in `to_markdown()` need to render enrichments? Almost certainly yes — when bundling to ponder, all enrichment context should be included.

---

### WHERE WE ARE / NEXT MOVE / COMMIT SIGNAL

**WHERE WE ARE:** Two improvements fully scoped. Edit is trivial. Enrichments introduce a new sub-model (`enrichments: Vec<Enrichment>`) with V1 manual + V2 agent-driven phases.

**NEXT MOVE:** Write spec artifact (`feedback-improvements` ponder) capturing data model, API contract, UX decisions. Then break into two features: `feedback-edit` and `feedback-enrich`.

**COMMIT SIGNAL:** Spec written and reviewed. At that point, commit to two features.

---

---
session: 1
timestamp: 2026-03-01T00:00:00Z
orientation:
  current: "Two distinct improvements scoped: inline edit and MCP-driven note enrichment. Both have clear implementation paths."
  next: "Write spec — capture data model changes (update fn + PATCH route + optional enrichments field), UX for inline edit, and enrichment flow (agent call on a note, append findings)."
  commit: "Spec written and design aligned. Proceed to feature backlog."
---

**jordan · Owner**
feedback improvements

we want to improve the feedback - should be able to edit/delete feedback, should be able to run an mcp call to add more information to feedback

---

## Session 1 — Scoping the two improvements

**Facilitator** opening the session with the brief. Three thought partners recruited: **Ben Hartley** (developer productivity UX), **Felix Wagner** (data model/lifecycle), and **Dan Reeves** (systems minimalist).

---

### What we're actually working with

Before diving in: the current feedback system is a simple append-only note queue. Here's what exists:

- `feedback.rs` — `add`, `list`, `delete`, `clear`, `to_markdown`. No `update`.
- Server routes — `GET /api/feedback`, `POST /api/feedback`, `DELETE /api/feedback/:id`, `POST /api/feedback/to-ponder`
- `FeedbackPage.tsx` — compose textarea + save, note cards with double-click-to-confirm delete, submit-all-to-ponder button
- `/__sdlc/feedback` — public tunnel alias (external agents can add notes)

**Delete already exists.** The stated requirement conflates two things: (a) delete is there, (b) **edit is missing**. So the first improvement is edit. The second is MCP enrichment.

---

### Improvement 1: Edit feedback notes

**Ben Hartley** weighs in first.

> "Inline edit is the right call here — don't open a modal. The note card already has the text in a `<pre>`. When the user clicks into the content area, swap to a textarea in-place, same styling. ⌘+Enter saves, Escape cancels. Delete stays on the card in non-edit mode. This is the GitHub inline comment edit pattern."
>
> "The trap to avoid: don't add an edit button as a separate affordance. Edit-on-click is discoverable enough for this audience (developers). A pencil icon in the top-right adds visual noise to every card."

**Dan Reeves** challenges the premise.

> "Wait — delete already exists. Edit is basically 'delete and re-create with corrected text.' Why do we need a mutation primitive? Is there actual evidence that users are making typos in quick-capture notes and then wanting to fix them rather than just deleting?"
>
> "Counter-argument from myself: the note has an ID (F1, F2, ...) that you might want to preserve for any downstream reference. If you delete-and-recreate, you get a new ID. That's a weak argument but it's the strongest one for edit."

**Felix Wagner** on the data model side.

> "The ID preservation argument is real but thin. The stronger case: if you're building toward enrichments (second improvement), you need edit to be a first-class mutation because enrichments will attach to a note ID. Delete-recreate breaks that link."
>
> "Data model for edit: trivially simple. `update(root, id, new_content)` in `feedback.rs` — load all, find by ID, replace content, atomic write. PATCH `/api/feedback/:id` with `{ content: string }` body. No new fields. Five minutes of Rust."

⚑  **Decided:** Edit is justified for ID preservation and forward-compatibility with enrichments. Inline edit UX (click-to-edit, not a button), ⌘+Enter to save, Escape to cancel.

---

### Improvement 2: MCP call to add more information

This is the more interesting requirement. Let's interrogate what "run an mcp call to add more information to feedback" actually means.

**Facilitator** proposes three interpretations:

**A.** Agent-driven enrichment: from the UI, trigger a Claude agent run that reads the codebase/project state and appends structured context to a note. E.g., note says "dashboard is slow" → agent reads telemetry/runs/advisory → appends: "See runs from Feb 28: 3 runs exceeded 30s. Likely cause: feature list SSE polling."

**B.** External tool injection: use a Playwright MCP call (or fetch) to pull in external information — e.g., link to a GitHub issue, doc page, error trace — and append it to the note.

**C.** MCP tool exposure: expose `POST /api/feedback` as an MCP tool descriptor so agents (in Claude Code or other contexts) can call it to add notes programmatically, possibly with richer metadata than the basic `content` string.

**Dan Reeves** cuts through.

> "C is already done — `/__sdlc/feedback` is exactly that. Don't reinvent it. Cross off C."
>
> "B and A are different at the invocation layer but similar at the data model layer. Both produce additional text that gets attached to an existing note. The question is: does the extra content live inside `content` (appended), or does it go into a new `enrichments: []` field on the note?"

**Ben Hartley** on UX implications.

> "If enrichments are appended into content, the note becomes a monolithic blob. The user loses the boundary between what they wrote and what the agent added. That boundary matters — it affects trust. You want to see 'here's what I wrote, here's what the agent found.'"
>
> "So: `enrichments: Vec<Enrichment>` where each has `source` (the tool/agent that added it), `content`, `added_at`. The card renders the original content, then below a subtle divider, enrichment blocks with a different visual treatment — muted background, maybe a small sparkle icon for agent-added content."

**Felix Wagner** on the data model decision.

> "Agreed on separate `enrichments` field. It also makes serialization clean — the base note stays stable, enrichments append. No rewriting the main content string."
>
> "For 'B — external URL/page pull': this is actually simpler than running a full agent. User pastes a URL into a text field on the note card, we fetch the page title + summary, append as an enrichment. No MCP protocol needed, just `POST /api/feedback/:id/enrich` with `{ url: string }` or `{ content: string, source: string }`."

**Dan Reeves** pushes back on enrichment complexity.

> "You're describing a new data sub-model (`enrichments` field + new route) for a feature that hasn't been validated. Start with the simpler path: `POST /api/feedback/:id/enrich` takes `{ content: string, source: string }` and appends to an `enrichments: Vec` on the note. That's it. No URL-fetch, no agent invocation inside the server — the agent runs externally (via `spawn_agent_run` or directly) and calls the enrich endpoint with pre-fetched content."
>
> "The MCP angle: the CLI or the server exposes an MCP tool `feedback_enrich` that takes `{ id, content, source }`. External agents (Claude Code running `/sdlc-run`) can call this while working on a feature — 'I found a relevant issue, let me enrich this feedback note.'"

⚑  **Decided:** `enrichments` as a separate field on `FeedbackNote`. Structure:
```rust
pub struct Enrichment {
    pub source: String,      // "agent:sdlc-run", "user:url", "mcp:fetch", etc.
    pub content: String,
    pub added_at: DateTime<Utc>,
}
```
Mutation: `POST /api/feedback/:id/enrich` with `{ content, source }`.

---

### The MCP tool angle

**Ben Hartley** surfaces the UX entry point question.

> "OK so the user wants to 'run an MCP call' — that phrase implies user agency, not agent-driven. They want to click something in the UI that triggers a research action. Not just 'agents can call this endpoint'; the user initiates it."
>
> "Proposal: each note card gets a small 'Research' action (⚡ icon). Clicking it opens a mini-prompt: 'What should the agent look for?' with a textarea. User types context, submits, agent runs (via `spawn_agent_run`), enrichment appears below the note content when done."

**Dan Reeves** on minimal footprint.

> "That's a full agent run per note. Cost and latency concerns for a quick-capture tool. Counter: start with manual enrichment. User clicks 'Add context', gets a textarea, types enrichment content themselves, saves it. The structured enrichment data model is in place. Agent-driven enrichment is an overlay on top — you flip the source from 'user:manual' to 'agent:run' but the storage layer is identical."

**Felix Wagner** on phasing.

> "Dan's right on sequencing. V1 of enrichment: manual `POST /api/feedback/:id/enrich` — user appends their own context (e.g., paste in a stack trace, link to a doc). V2: agent-driven enrichment via spawn_agent_run. The MCP tool (`feedback_enrich`) is V1-compatible — external agents can call it immediately."

⚑  **Decided:** Two-phase enrichment:
- **V1**: Manual enrichment — `POST /api/feedback/:id/enrich` with `{ content, source: "user" }`. UI: "Add context" button on note card. Text field opens below note, user types, saves. Source tagged as "user".
- **V2**: Agent-driven enrichment — UI prompt triggers `spawn_agent_run`, result POSTs back via enrich endpoint. Source tagged as "agent:research".

---

### Summary of scope

**Improvement 1 — Edit** (small, self-contained):
- `update(root, id, new_content) -> Result<FeedbackNote>` in `feedback.rs`
- `PATCH /api/feedback/:id` with `{ content: string }` body
- `api.updateFeedbackNote(id, content)` in client.ts
- Inline edit in `NoteCard`: click content area → textarea, ⌘+Enter saves, Escape cancels

**Improvement 2 — Enrichments** (medium, new sub-model):
- Add `Enrichment { source, content, added_at }` struct to `feedback.rs`
- Add `enrichments: Vec<Enrichment>` field to `FeedbackNote` (default empty)
- `enrich(root, id, source, content) -> Result<FeedbackNote>` function
- `POST /api/feedback/:id/enrich` with `{ content, source }` body
- `api.enrichFeedbackNote(id, content, source)` in client.ts
- Note card renders enrichment blocks below a divider, muted styling
- "Add context" button on hover expands an inline textarea for manual enrichment entry
- MCP tool descriptor: `feedback_enrich` registered in sdlc MCP server

**?  Open:** Should V2 agent-driven enrichment be scoped into the same feature or a follow-on? Lean toward follow-on — the data model + manual UI is a complete unit.

**?  Open:** Does `FeedbackNote` in `to_markdown()` need to render enrichments? Almost certainly yes — when bundling to ponder, all enrichment context should be included.

---

### WHERE WE ARE / NEXT MOVE / COMMIT SIGNAL

**WHERE WE ARE:** Two improvements fully scoped. Edit is trivial. Enrichments introduce a new sub-model (`enrichments: Vec<Enrichment>`) with V1 manual + V2 agent-driven phases.

**NEXT MOVE:** Write spec artifact (`feedback-improvements` ponder) capturing data model, API contract, UX decisions. Then break into two features: `feedback-edit` and `feedback-enrich`.

**COMMIT SIGNAL:** Spec written and reviewed. At that point, commit to two features.
