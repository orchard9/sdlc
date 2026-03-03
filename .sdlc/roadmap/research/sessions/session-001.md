---
session: 1
timestamp: 2026-03-03T03:00:00Z
orientation:
  current: "Idea fully shaped — three gaps identified and solved. Backend needs web search tools + better prompt. Frontend needs a 'New Research' modal on the list view."
  next: "Create two features: (1) knowledge-research-web-tools — add WebSearch/WebFetch to research agent and rewrite prompt; (2) knowledge-research-modal — add NewResearchModal to knowledge list header."
  commit: "Met — implementation shape is unambiguous and scoped. Ready to crystallize into features."
---

**jordan · Owner**

> research
>
> our knowledge page at /knowledge needs a way to be able to issues research jobs that are very similar to the evolve job found at ~/Workspace/orchard9/adk-go-researcher

---

**Facilitator orientation:**

Full archaeological read of both the adk-go-researcher and the sdlc knowledge infrastructure before the session. Here's what actually exists:

**Already built in sdlc:**
- `POST /api/knowledge/{slug}/research` endpoint (creates entry if missing, spawns agent)
- "Research More" button on entry detail pane
- `KnowledgeResearchStarted` / `KnowledgeResearchCompleted` SSE events
- `api.researchKnowledge(slug, topic?)` client method
- `ResearchKnowledgeBody` with optional `topic` field

**The adk-go-researcher architecture:**
- Multi-agent pipeline via ADK-Go: Planner → Researcher (with Google Search) → Writer
- Event-driven: streams `started`, `phase_start`, `finding`, `completed` events
- Both HTTP/SSE and CLI interfaces

**Critical observation:** The gaps are smaller than they appear. Infrastructure is in place. Two targeted fixes + one new component.

---

**Priya Nair (Distributed Systems):**

The infrastructure is already correct — `spawn_agent_run` handles everything. The gap is:

1. **Tools missing from research agent**: `sdlc_query_options` doesn't include `WebSearch`/`WebFetch`. Compare to `sdlc_guideline_query_options` (line 609, runs.rs) which extends the base with web tools for the Prior Art Mapper. Same pattern needed for knowledge research. Create `knowledge_research_query_options()`.

2. **Prompt says "grep the codebase"**: `build_research_prompt` in knowledge.rs tells the agent to use Grep and Read on local files. It needs to use WebSearch + WebFetch to pull external knowledge.

These are two surgical changes, not architectural work.

⚑  **Decided:** Backend fix = `knowledge_research_query_options()` + rewritten prompt. Two functions touched.

---

**Ben Hartley (Developer Productivity UX):**

The "Research More" button is deeply buried: knowledge page → select entry → detail pane → find button. Three clicks to reach a first-class action. Jordan's word "issue research jobs" signals this should feel more prominent — like a top-level operation, not a secondary detail action.

**Proposed UX:**
- "Research" button in the knowledge list header (visible from list view without selecting an entry)
- Simple modal: topic field + optional slug preview (auto-slugified)
- Submit → backend creates entry if needed + starts agent → navigate to entry → show "researching" state
- Works for both new entries (topic → new slug) and existing entries (same topic → finds existing slug)

The `api.researchKnowledge(slug, topic)` already handles both cases because the backend creates the entry if the slug doesn't exist.

?  **Open:** Should the modal have a "Category" (knowledge code) selector so the new entry goes into the right catalog division? Probably yes for V1, since uncategorized entries are hard to find later.

---

**Dana Cho (Product Skeptic):**

Gap inventory — what's missing vs what exists:

| Capability | Status |
|---|---|
| Backend endpoint | ✅ exists |
| SSE events | ✅ exists |
| API client method | ✅ exists |
| "Research More" on existing entry | ✅ exists |
| Web search tools in research agent | ❌ missing |
| Better research prompt (web-aware) | ❌ missing |
| "Start new research" from list view | ❌ missing |

Minimum viable delta: 2 backend changes + 1 new modal component.

Do NOT add depth selector in V1 — adds UI complexity without clear value. Agent decides depth based on topic.

Do NOT auto-publish entries when research completes — knowledge entries should be reviewed before publishing.

⚑  **Decided:** Scope = web tools + better prompt + new modal. Everything else is V2.

---

**Tobias Krenn (Skeptical Engineering Lead):**

Two risks to flag:

1. **The category question is real** — new entries default to "uncategorized" which makes them invisible in catalog navigation. The modal should include a code/category field (could default to "uncategorized" but make it editable). The catalog is pre-loaded on the page so we can render a dropdown.

2. **Prompt quality gap**: Current prompt ends with "Only include information you actually found in the codebase or can verify." That's backward for web research — we WANT external sources. The new prompt should say "Search the web for authoritative sources. Prefer official docs, academic papers, and established technical references."

⚑  **Decided:** Modal includes optional category code dropdown using catalog data already loaded on the page.

---

**Facilitator synthesis:**

### Implementation Plan

**Feature 1: `knowledge-research-web-tools` (backend)**

File: `crates/sdlc-server/src/routes/knowledge.rs`

1. Add `knowledge_research_query_options(root, max_turns)`:
   ```rust
   fn knowledge_research_query_options(root: PathBuf, max_turns: u32) -> QueryOptions {
       let mut opts = sdlc_query_options(root, max_turns);
       opts.allowed_tools.push("WebSearch".into());
       opts.allowed_tools.push("WebFetch".into());
       opts
   }
   ```

2. Update `research_knowledge` to use it:
   ```rust
   let opts = knowledge_research_query_options(app.root.clone(), 30);
   ```

3. Rewrite `build_research_prompt` with web-first instructions:
   - Plan what to research (3-5 key questions)
   - Use WebSearch for each question (5-10 searches)
   - Use WebFetch to read 3-5 most promising pages
   - Synthesize into structured Markdown
   - Update entry summary + write content.md
   - Log session

**Feature 2: `knowledge-research-modal` (frontend)**

File: `frontend/src/pages/KnowledgePage.tsx`

1. Add `NewResearchModal` component:
   - Topic field (required)
   - Category code select (optional, defaults to "uncategorized", uses catalog data)
   - Slug preview (auto-generated, editable)
   - Submit → `api.researchKnowledge(slug, topic)` → navigate to `/knowledge/${slug}`

2. Add "Research" button in `KnowledgePage` list header area next to the "Knowledge / Catalog" title.

3. On `KnowledgeResearchCompleted` SSE, refresh the entry list.

### The research prompt sequence (adk-go-researcher analogy)

```
1. PLAN: What are the 3-5 key questions this topic raises?
2. SEARCH: WebSearch each question (5-10 queries total)
3. READ: WebFetch the 3-5 most relevant pages
4. SYNTHESIZE: Write structured Markdown (overview, key concepts, gotchas, references)
5. SAVE: Update summary + write content.md
6. LOG: sdlc knowledge session log {slug}
```

This maps to the adk-go-researcher's Planner → Researcher → Writer pipeline — just in a single Claude agent with explicit phase instructions instead of separate ADK-Go agents.

---

⚑  **Decided:** Implementation shape is complete and unambiguous. Ready to crystallize.
