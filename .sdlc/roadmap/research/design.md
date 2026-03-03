## Design: Knowledge Research Jobs

### What Already Exists
- `POST /api/knowledge/{slug}/research` endpoint
- `KnowledgeResearchStarted` / `KnowledgeResearchCompleted` SSE events
- 'Research More' button on entry detail pane
- `api.researchKnowledge(slug, topic?)` client method

### Gap 1: No Web Search Tools
The research agent uses `sdlc_query_options` which does NOT include WebSearch/WebFetch.
Fix: create `knowledge_research_query_options()` extending base with WebSearch + WebFetch.
Pattern: same as `sdlc_guideline_query_options` at line 609 in runs.rs.

### Gap 2: Prompt Only Does Local Search
Current `build_research_prompt` tells Claude to grep the codebase.
Fix: rewrite to use WebSearch, follow Planner→Researcher→Writer sequence.

### Gap 3: No 'Start Research' from List View
'Research More' is buried in entry detail. No way to kick off research on a new topic.
Fix: 'Research' button in list header → NewResearchModal → topic + slug → submit.

### Implementation Shape
**Backend** (knowledge.rs):
1. `knowledge_research_query_options()` — extends sdlc_query_options with WebSearch + WebFetch
2. Use it in `research_knowledge` instead of `sdlc_query_options`
3. Rewrite `build_research_prompt` with web search instructions

**Frontend** (KnowledgePage.tsx):
4. 'Research' button in list header area
5. `NewResearchModal` — topic field, auto-slugified, submit
6. Navigate to entry on submit so user sees researching state

### Research Prompt Shape
1. Analyze the topic, plan what to research
2. Use WebSearch to find 5-10 authoritative sources
3. Use WebFetch to read the most promising ones
4. Synthesize into structured Markdown (overview, key concepts, references)
5. Update knowledge entry: `sdlc knowledge update {slug} --summary "..."`
6. Write content to entry: content.md via Write tool
7. Log session: `sdlc knowledge session log {slug} --content "..."`