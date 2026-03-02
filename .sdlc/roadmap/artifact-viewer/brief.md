# Artifact Viewer — Rich Artifact and Plan Viewer

**Origin:** Extracted from Discord conversation dump (sdlc early-user feedback session)

**Summary:**
Xist repeatedly hits a wall trying to read plan artifacts in the SDLC UI. The current viewer is too small to read. He works around it by exporting plans to `.md` files and reading them in Agy, which has a rich rendered Markdown viewer with file/class links. This is the #1 differentiator Xist sees between Agy and SDLC — and it's the thing he most wants SDLC to match or beat. The request goes beyond display: Xist wants Google Doc-style inline commenting so he can annotate plans and submit all feedback as one LLM request.

**Key signals (all strong):**
- [Product/User] "I can't really see that." — referring to the artifact expand view
- [Product/User] "What I'm doing that is working great, is I'm telling the agents 'save the plan as Plans/foo.md' and then I open Plans/foo.md in a Markdown previewer in Agy." — workaround reveals the gap
- [Product/User] "TLDR I don't want to have to read the entire convo every time to know what is happening. I just want to see the updated plan." — summary-after-agent-run request
- [Product/User] "Long term ideally the 'Plan Viewer' (Artifact Viewer more generally) works like a Google Doc. User clicks on the page, adds comments. Add 3, 4 comments at a time about different things in the plan. Then ONE TIME submit all the comments for the agents to work on." — consolidated commenting
- [Product/User] "An AMAZING feature of Agy and what really makes it shine is this planning capability." — Agy as benchmark

**Relevant excerpts (verbatim):**
> "I think this is the only way I can see artifacts currently? [Image] I can't really see that."

> "What I'm doing that is working great, is I'm telling the agents 'save the plan as Plans/foo.md' And then I open Plans/foo.md in a Markdown previewer in Agy. I always make Agy include links in my plans, so all source files/classes named in the plans are linked to their actual source location and I can open them up in Agy and look at them. Amazing during the planning process."

> "Most important aspect: Agents think for 15 mins, TONS of output. I want at the end of that output: Summary of new plan as revised. Optional ability to see all the current output, the details that went into preparing the plan."

> "User clicks on the page, adds comments. Add 3, 4 comments at a time about different things in the plan. Then ONE TIME submit all the comments for the agents to work on. Much less time wasted for the user -- consolidate all comments into 1 single LLM request."

> "Jordan lmk if you want me to give you a demo of how I do this in Agy so you can see my workflow."

> "I'll use it to do a planning session for a new project today, I'll record it and then put the recording on speedup. Screenshots don't do it justice. The flow is what is nice about it. (If only it ran faster!)"

**Open questions:**
- Should the artifact viewer be a panel within the feature page, or a separate full-screen mode?
- Is rendered Markdown enough, or do we need live file-link resolution (links that open IDE)?
- What is the "summary after agent run" mechanism — agent-generated TLDR, or structured artifact output?
- Does Google Doc-style commenting require a new data model (comments per artifact), or can we layer it on the existing artifact system?
- How does this relate to the ponder scrapbook — is that a different viewer, or the same one?
