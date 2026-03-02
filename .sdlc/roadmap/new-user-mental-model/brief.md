# New User Mental Model — Guided Workflow and Discovery for First-Time Users

**Origin:** Extracted from Discord conversation dump (sdlc early-user feedback session)

**Summary:**
Xist's first session revealed a significant gap between how Jordan thinks about using SDLC and how a new user experiences it. Jordan's flow (ponder → plan → converge → commit → run wave) is never taught. The dashboard gatekeeps "Run Wave" behind Vision + Architecture setup, but doesn't explain what those are or why they're needed. Xist started 20+ features individually because he didn't know about Run Wave. The discoverability of the flow is entirely by observation or word-of-mouth. This isn't just an onboarding problem — it's a mental model problem. Jordan and Xist are operating from completely different understandings of the tool's intended flow.

**Key signals (all strong):**
- [Product/User] "I do not know how to use this tool!" — direct statement, 2x in session
- [Product/User] "I put together this big plan. It broke it down into lots of pieces. It wants me to personally execute each piece." — didn't know about Run Wave or autonomous execution
- [Product/User] "No, I'm a noob I didn't set up my Vision, Architecture yet, Dashboard just tells me to do that." — setup dependency unexplained
- [Process] Jordan sharing his mental model: "my flow is ponder -> plan -> converge -> commit -> run wave then while the wave is running ponder -> ponder ->" — this flow is never surfaced in the UI
- [Culture] "The only way to really understand what it's doing is to watch it while it works and read its output. You've done that so much you don't really need to anymore, you're thinking bigger picture. Every new dev is going to stare at this screen until they are comfortable that they understand what it is doing."

**Relevant excerpts (verbatim):**
> "I do not know how to use this tool!"

> "I put together this big plan. It broke it down into lots of pieces. It wants me to personally execute each piece. I thought it would like run them all in order or something, since they're all in the same plan."

> "No, I'm a noob I didn't set up my Vision, Architecture yet, Dashboard just tells me to do that. I guess I'll do that so I can see the dashboard"

> "my flow is ponder -> plan -> converge -> commit -> run wave. then while the wave is running: ponder -> ponder -> ponder -> then i just check in on waves, tests, and adjust with 'fix right away'"

> "The only way to really understand what it's doing is to watch it while it works and read its output. You've done that so much you don't really need to anymore, you're thinking bigger picture. Every new dev is going to stare at this screen until they are comfortable that they understand what it is doing."

> "IDK the best way to initialize this stuff. I feel like it keeps stopping, maybe because it wants my input. But I just want it to build everything and then I'll look at the final result and iterate on that."

**Open questions:**
- What is the intended onboarding flow from the perspective of someone who's never seen SDLC?
- Should the UI show the "flow" (ponder → plan → commit → run wave) as a visible pipeline?
- How do we teach the ponder → run wave relationship without requiring Jordan to explain it each time?
- Is the Vision/Architecture gate the right first step, or should we allow people to skip into "explore mode" first?
- What does "I just want it to build everything" tell us about what Xist's mental model of the tool should be?
- How does the "watching vs. fire-and-forget" dichotomy affect the UI design — can we support both modes?
