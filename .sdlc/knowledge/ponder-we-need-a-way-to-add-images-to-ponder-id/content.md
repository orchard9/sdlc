---
session: 1
timestamp: 2026-03-02T18:42:26Z
orientation:
  current: "Four distinct asks: bigger creation modal, artifact attachment, URL references, ponder agent tools (web search + playwright). Item 4 is the most technically consequential — it changes what the ponder agent can *do*, not just what users can input."
  next: "Build the creation modal first (1-day win, highest UX leverage). Then add WebSearch/WebFetch to ponder agent (2 lines). Then URL reference field. Then design binary attachment strategy."
  commit: "When we have clear scope boundaries on binary attachment storage and a concrete spec for the expanded creation modal."
---

**jordan · Owner**
we need a way to add images to ponder ideas

1. the creation of a ponder idea should be done in a bigger modal with a larger space for descriptions
2. we should be able easily attach artifacts
3. we should be able to reference sites
4. we need to make sure the mcp server has access to web search and can use playwright to take its own screenshots

---

**Facilitator**
Four distinct asks. Let me walk through what we're actually building, what it costs, and where the real leverage is. I'm going to pull in Ben Hartley (developer UX), Tobias Krenn (scope skeptic), and Nadia Osei (document experience) because this brief touches all three of their wheelhouses.

Let's start with the real problem underneath the requests.

---

**Ben Hartley · Developer Productivity UX**
The creation flow is the wrong shape for what it's supposed to do. Right now, `NewIdeaForm` is an inline panel in the sidebar — 2-row textarea, compact slug input, no room to think. The brief is the *seed* for a 100-turn agent session. You're asking users to plant a rich idea in a thimble.

The psychological issue: small input boxes produce small inputs. A 2-row textarea signals "this should be short." But a ponder brief should be the opposite — expansive, context-rich, everything you can think of before the agent takes over. The form factor is actively suppressing the quality of the seed material.

What I'd build: a proper full-screen dialog (or at minimum an 80vw modal) with a large auto-growing textarea. Title at the top, brief taking up the bulk of the space, slug auto-derived. The "Create" button should feel like a launch button, not a form submit.

The key UX decision: does creating an idea immediately fire the chat session (current behavior), or does it create the entry and let you land on the ponder page first? I'd argue: keep the immediate fire — the modal closing and the agent starting is satisfying. But give users 3 seconds to see the entry before the chat indicator appears.

⚑ Decided: Creation modal should be a proper dialog (not inline sidebar form), with an auto-growing textarea that signals "give me the full picture."

---

**Tobias Krenn · Skeptical Engineering Lead**
I want to scope-check item 2 ("attach artifacts") before we go any further. This one has a failure mode I've watched happen three times.

"Attach artifacts" sounds like "drag a file, it appears." In reality it means: file input handling, multipart upload, MIME type validation, storage strategy (disk? base64-in-yaml? attachment directory?), serving the files back, rendering them in the artifact viewer, and making sure the agent can actually *read* them. That's a full sprint, minimum.

The markdown-text-to-scrapbook pipe already exists: `POST /api/roadmap/:slug/capture` with `{ filename, content }`. The question is whether "content" can be base64-encoded binary data or not. Right now the server writes it as a string to disk — there's nothing stopping base64 in theory, but nothing rendering it on the other side either.

My recommendation: kill binary attachment for now. Support two things instead:
1. Paste a URL → stored as a reference artifact with the URL + auto-fetched page title
2. Paste markdown text → stored as a text scrapbook artifact (already works)

If there's a validated use case for drag-to-upload images after 30 days of usage, revisit. Right now we're solving a hypothesis, not a confirmed pain point.

? Open: Does jordan specifically need binary image upload, or would "paste a URL and have the agent screenshot it" satisfy the need?

---

**Nadia Osei · Document Experience**
I want to reframe items 3 and 4 together, because they're actually the same ask seen from two different angles.

Item 3: "reference sites" — the user wants to attach context from the web to an idea.
Item 4: "playwright screenshots" — the *agent* wants to pull context from the web into the idea.

These are two sides of the same capability: **web context as ponder input**. The difference is who initiates the fetch — the human (paste a URL in the creation form) or the agent (take a screenshot mid-session).

For the human side: add a "Reference URLs" field to the creation modal. Multiple URLs, one per line. When the idea is created, capture them as a `references.md` scrapbook artifact. Optionally, auto-fetch the page title (the API already uses `ureq` for exactly this in the knowledge base). Simple, no risk.

For the agent side: WebSearch and WebFetch are already in `sdlc_query_options` for guideline investigations — they just aren't pushed into ponder chat. That's literally 2 lines of code:
```rust
opts.allowed_tools.push("WebSearch".into());
opts.allowed_tools.push("WebFetch".into());
```

Playwright for ponder: the pattern is established (UAT runs, lines 498-511). Adding it to ponder runs is ~10 lines. The agent can take a screenshot, get the image data back, and… then what? The agent needs to store it somewhere. It can write a markdown artifact with a note like "Screenshot taken of X at Y time — image data not storable as text." Or we accept that the agent uses `browser_take_screenshot` with a filename and we find a way to serve those files.

Actually — wait. The `browser_take_screenshot` tool saves to a file path. If the agent saves to `.sdlc/roadmap/<slug>/screenshot-N.png`, the server could serve it as a static attachment. That's the simplest path that doesn't require base64 encoding.

⚑ Decided: Web search + WebFetch → add to ponder chat runs immediately (2 lines). Playwright → add with the understanding that screenshots get stored as `.png` files in the ponder scrapbook directory and served statically.

---

**Ben Hartley · Developer Productivity UX**
On the screenshot storage point — Nadia is right that the agent saving to a filesystem path is the simplest model. But we need the UI to render those screenshots in the WorkspacePanel/DialoguePanel. Currently, artifact viewing assumes text content. PNG artifacts need either:
- An `<img src="/api/roadmap/:slug/artifacts/:filename">` endpoint (need a static file serve route), or
- The agent embeds a markdown image `![](...)` with a relative path

The markdown embed approach is elegant because WorkspacePanel already renders markdown. The agent writes `![screenshot](./screenshot-1.png)` in a `.md` artifact, and the markdown renderer tries to load the image. As long as `/api/roadmap/:slug/artifacts/screenshot-1.png` is routed, it works.

? Open: Does the server currently have a route to serve arbitrary files from the ponder scrapbook directory?

---

**Tobias Krenn · Skeptical Engineering Lead**
Let me answer that: No, the server does not. The `GET /api/roadmap/:slug` endpoint reads artifacts as text strings (`read_artifact` returns a String). There's no binary file serving. To add it: a new route `GET /api/roadmap/:slug/file/:filename` that reads bytes from disk and serves with the correct Content-Type.

That's one day of Rust work. Worth it if playwright screenshots are a real use case.

But here's the thing: jordan said "we need to make sure the mcp server has access to web search and can use playwright to take its own screenshots" — this is about what the *agent* can do, not necessarily about rendering those screenshots back in the UI. The agent's use case might be: take a screenshot to understand a site, then describe what it saw in text. The screenshot doesn't need to be stored or displayed. The agent's output (a text summary) gets logged in the session.

So there are two sub-modes here:
1. **Agent uses playwright to understand + describe** — no storage needed, just add playwright tools
2. **Agent takes screenshots that users can see in the UI** — needs file serving + image rendering

Start with mode 1. It's already useful and costs nothing extra beyond adding playwright to the ponder run.

⚑ Decided: Playwright for ponder runs → add tools to `start_ponder_chat` following UAT pattern. Screenshots go to `/tmp/` for agent use; not persisted by default. Binary storage is a follow-up decision.

---

**Facilitator**
Let me pull together the decisions so far and identify the four concrete deliverables:

**Deliverable 1: Expanded Creation Modal (1–2 days)**
- Replace `NewIdeaForm` inline sidebar with a proper modal dialog
- Auto-growing textarea for brief (min ~8 rows, grows to fill)
- "Reference URLs" field — multi-line or repeated inputs, one URL per line
- On submit: capture URLs as `references.md` scrapbook artifact
- Slug auto-derived from title (existing behavior)
- Keep the immediate chat-start behavior (no new loading state needed)

**Deliverable 2: WebSearch + WebFetch for Ponder Agent (2 lines)**
- In `start_ponder_chat` (runs.rs:924), add:
  ```rust
  opts.allowed_tools.push("WebSearch".into());
  opts.allowed_tools.push("WebFetch".into());
  ```
- Done. Agent can now research sites, fetch page titles, read documentation.

**Deliverable 3: Playwright for Ponder Agent (~15 lines)**
- In `start_ponder_chat`, add the playwright MCP server config and tool set following the same pattern as `start_milestone_uat` (lines 498-511)
- Agent stores screenshots in `/tmp/` for its own use, describes findings in text
- Does NOT require binary file serving in this iteration

**Deliverable 4: URL Reference in Creation Modal (included in D1)**
- Part of the expanded creation modal — URL field captures references immediately
- Agent can use those URLs as starting points for WebFetch research on session start

? Open: "Attach artifacts" beyond URLs — what format does jordan need? Documents? Images? Code snippets? This needs one clarifying question before building. Best guess: paste a code snippet or paste markdown text → captured as a scrapbook file. That already works via `capturePonderArtifact`. Needs a UI entry point.

---

**Ben Hartley · Developer Productivity UX**
On "attach artifacts" via the UI: the simplest model is a "Drop text / paste here" zone below the brief textarea in the creation modal. Users paste a block of text (code snippet, reference doc, partial spec) and it gets captured as a `context-N.md` scrapbook file alongside the brief. That's zero new backend work — it just calls `capturePonderArtifact` with the pasted content.

For images: paste-from-clipboard is actually doable with `ClipboardEvent` + `FileReader` → base64 data URI → store as PNG data URI in a markdown image tag. Renders without a file server. But it bloats the file significantly for large images. Acceptable if images are small (screenshots, diagrams); not acceptable if images are photos.

Scope decision: **support clipboard image paste** in the creation modal as a markdown `![](data:image/png;base64,...)` embed, stored in a `context-N.md` scrapbook artifact. The agent can read the data URI if needed, but it's primarily for human reference.

? Open: Tobias, do you accept clipboard image paste as in-scope? It's purely client-side — no new API surface.

---

**Tobias Krenn · Skeptical Engineering Lead**
Clipboard image paste is fine if it's purely client-side. No new routes, no new storage format changes in core. The output is still a markdown string that goes through `capturePonderArtifact`. I accept that scope.

⚑ Decided: Image attachment = clipboard paste → base64 data URI in markdown → stored via `capturePonderArtifact`. Agent-visible if needed. No new backend work.

---

**Nadia Osei · Document Experience**
The creation modal layout, to be concrete:

```
┌─────────────────────────────────────────────────────────────┐
│  New Idea                                          [×]       │
├─────────────────────────────────────────────────────────────┤
│  What are you thinking about?                               │
│  ┌─────────────────────────────────────────────────────┐   │
│  │ [Title input — full width]                          │   │
│  └─────────────────────────────────────────────────────┘   │
│  Slug: [auto-derived, editable]                             │
│                                                             │
│  Brief (describe the idea fully — this seeds the agent)     │
│  ┌─────────────────────────────────────────────────────┐   │
│  │                                                     │   │
│  │ [Auto-growing textarea, min 8 rows]                 │   │
│  │                                                     │   │
│  │ Paste images or text here ↑                         │   │
│  └─────────────────────────────────────────────────────┘   │
│                                                             │
│  Reference URLs (one per line, optional)                    │
│  ┌─────────────────────────────────────────────────────┐   │
│  │ https://...                                         │   │
│  │                                                     │   │
│  └─────────────────────────────────────────────────────┘   │
│                                                     [Cancel] │
│                                               [Create Idea ▸]│
└─────────────────────────────────────────────────────────────┘
```

Width: 2xl (672px) or max-w-3xl. Backdrop blur. No tabs inside the modal — one flat form is correct here.

⚑ Decided: Modal layout as above. Paste target is the brief textarea itself (paste event listener). URLs are a separate field below.

---

**Facilitator**
Summary of decisions and open questions:

## Decisions

1. **Creation modal** → Replace inline sidebar form with a 2xl modal. Auto-growing brief textarea (min 8 rows). Reference URLs field. Paste images into the brief textarea using clipboard API → base64 data URI stored as markdown scrapbook artifact.

2. **WebSearch + WebFetch for ponder agent** → 2 lines in `start_ponder_chat`. Ship immediately.

3. **Playwright for ponder agent** → Add MCP server config + tool list to `start_ponder_chat` following UAT pattern. Screenshots to `/tmp/` for agent reasoning, not persisted.

4. **URL references** → Dedicated field in creation modal. Captured as `references.md` scrapbook artifact on create. Agent can WebFetch them automatically.

5. **Binary file attachment** → Out of scope for this iteration. Clipboard paste (client-side base64) covers the primary image use case without new backend work.

## Open Questions

- Does jordan want clipboard paste or drag-and-drop file upload? (They said "add images" — clipboard paste satisfies this without backend work; file upload requires new routes)
- Should the agent automatically WebFetch reference URLs when a session starts? Or only on demand?

## Risk Flag

Playwright in ponder agent: the ponder session uses `max_turns: 100`. Playwright actions count as turns. Heavy browser sessions could exhaust the turn budget. Mitigation: the agent is instructed to use playwright judiciously, not as primary research mode.
