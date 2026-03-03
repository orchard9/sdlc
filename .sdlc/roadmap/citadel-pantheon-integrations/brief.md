# Brief: Citadel and Pantheon Integration Tools

**Origin:** Extracted from team conversation on 2026-03-02

## Summary

jx12n is planning to build integration tools for Citadel and Pantheon as the next workstream. The Citadel use case is concrete: fetch logs, annotate logs, and create tickets based on log content. Pantheon was mentioned alongside Citadel as the first integration targets.

## Key Signals

- **STRONG** [Product] Tools to fetch logs from Citadel
- **STRONG** [Product] Tools to annotate logs in Citadel
- **STRONG** [Product] Tools to create tickets based on log content
- **STRONG** [Strategy] Pantheon integration is also in scope — specifics not yet defined

## Relevant Excerpts

> jx12n: "i'll be working through integrations to pantheon and citadel first tomorrow - so tools to interact with citadel to fetch logs, annotate logs, create tickets based off logs"

## Open Questions

- What is Citadel? Log aggregation system? Observability platform? Internal tool?
- What is Pantheon? CMS? Hosting platform? Internal system?
- What does "annotate logs" mean — tagging them with metadata, adding comments, linking to code?
- What does "create tickets" mean — GitHub issues? Internal ticketing? Jira?
- Are these tools for the `sdlc` CLI, or for another project (community-api)?
- Should these be implemented as `.sdlc/tools/` entries or as first-class CLI commands?
- Is there an existing API for Citadel, or does this require scraping/export?
