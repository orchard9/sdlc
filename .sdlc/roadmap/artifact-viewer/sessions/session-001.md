---
session: 1
timestamp: 2026-03-01T00:00:00Z
orientation:
  current: "Raw signal from first real-user session — Xist's workaround (export to .md, view in Agy) defines the gap precisely"
  next: "Interrogate the brief: what is the minimum viewer that would eliminate the workaround, and what is the maximum vision?"
  commit: "Clear feature spec for artifact viewer with: rendering requirements, commenting model, summary-after-run mechanism, and relationship to ponder scrapbook"
---

## Session 1: Initial Signal Load

Bootstrapped by /sdlc-convo-mine from Discord conversation (sdlc early-user feedback session).

### Signals extracted

- **The workaround reveals the gap**: Xist tells agents to save plans to `Plans/foo.md` then opens them in Agy. He adds file/class links manually. This means: rendered Markdown is the minimum viable fix; but links that open the IDE is the next level.
- **TLDR after long runs**: "Agents think for 15 mins, TONS of output. I want at the end of that output: Summary of new plan as revised." This is a structured output requirement, not just a display requirement — agents need to emit a summary artifact.
- **Google Doc commenting**: 3-4 comments on different parts of the plan, submit all at once. This is the highest-complexity request but also the highest-value differentiator from any other tool.
- **Agy as benchmark**: Xist says "An AMAZING feature of Agy and what really makes it shine is this planning capability." He offered to demo it. That demo should happen.

### Why this might matter

This is the #1 item that keeps Xist in Agy for planning work. If SDLC has a comparable or better artifact viewer, Xist's full workflow moves to SDLC. It's also the most visible gap for any new user — the first thing anyone does is try to read what the agents produced.

### Open questions

- Does "rendered Markdown" mean in-panel, or does it need a full-screen mode?
- Is the TLDR a separate agent-generated artifact, or a structured field on the run record?
- Can Google Doc commenting be scoped to "inline comments on a plan Markdown file" as an MVP?
- What's the relationship between this and the ponder scrapbook viewer — same component?

### Suggested first exploration

Get the Agy demo from Xist. Understand specifically: what does the planning flow look like from his perspective, what makes it "amazing," and where SDLC falls short by comparison. Then design a response.
