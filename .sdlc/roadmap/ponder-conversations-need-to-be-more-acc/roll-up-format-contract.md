# Roll-Up Format Contract

## The Problem (reframed)

Session logs are **event logs** (sequential transcripts). Product people need **documents** (structured by importance). The issue isn't jargon alone — it's that 60% of session content is operational overhead (tool calls, agent scaffolding), not thinking.

## The Solution: Authoring Convention

Every ponder session log MUST end with a structured `## Product Summary` section. This is an authoring contract, not infrastructure.

## Format Schema

```markdown
## Product Summary

### What we explored
[1-2 sentences in plain English, no technical terms — what problem or idea this session covered]

### Key shifts
[What changed vs. before this session — decisions made, assumptions revisited, prior beliefs updated]

### Implications
[What this means for the feature/milestone, framed for someone who manages the backlog — plain product language, no tech jargon]

### Still open
[The 1-2 questions we didn't resolve, phrased as decisions that need to be made, not as technical investigations]
```

## Rules
- H3 subsection labels are **stable and locked** — agents cannot rename them
- Content under each H3 is free prose
- Implications section must connect findings to product/backlog consequences in plain language
- This section is what gets surfaced on session cards in the UI

## Implementation Path

**Now (Option A — Convention only):**
- Update the `/sdlc-ponder` skill instruction to require this format
- All future sessions include the section

**Next milestone (Option C — UI extraction):**
- Session list API extracts `## Product Summary` content as a `preview` field
- Frontend shows summary on session cards (not just 'Session N')
- Click-through reveals full session log

## Secondary Benefit
Structured summaries make session history **queryable and citable**. Knowledge librarian can index Implications sections across all sessions. Future agents can answer 'what did we decide about X?' without reading full logs.
