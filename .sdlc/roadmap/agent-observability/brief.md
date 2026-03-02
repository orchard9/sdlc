# Agent Observability — Agent Activity Monitor and Quota Visibility

**Origin:** Extracted from Discord conversation dump (sdlc early-user feedback session)

**Summary:**
Xist can see token cost ($) but doesn't know what it means in terms of quota. More importantly, he can't see what an agent is actually doing at any moment — is it waiting on LLM? Waiting on a sub-agent? Network? This is the bottleneck visibility problem. He wants something like macOS Activity Monitor: a real-time view of all active agents, what they're blocked on, their resource usage, graphed over time. This directly connects to Jordan's planned OTEL spike (mcp-run-telemetry), and also feeds into the concurrency question ("where can I add more parallelism?").

**Key signals (all strong):**
- [Product/User] "I find myself wondering: what is each agent doing? network? cpu? llm wait? sub-agent wait? etc" — precise breakdown of what's needed
- [Product/User] "what is happening to my quota? I see $ but I don't know what that means as far as quota usage" — cost visibility gap
- [Product/User] "Ideally in addition to seeing a snapshot of this, would love to see the same thing graphed over time for the last X amount of time starting whenever I first open the tool (like Activity Monitor in OS)" — time-series view
- [Product/User] "Where can concurrency be added, is another big question related to and based on this info." — connects to parallel execution design
- [Engineering] Jordan is building sdlc-spike for mcp-run-telemetry to address exactly this: "track all agent/subagent activity per run in a local embedded database"

**Relevant excerpts (verbatim):**
> "I find myself wondering: what is each agent doing? network? cpu? llm wait? sub-agent wait? etc. what is happening to my quota? I see $ but I don't know what that means as far as quota usage"

> "Ideally in addition to seeing a snapshot of this, would love to see the same thing graphed over time for the last X amount of time starting whenever I first open the tool (like Activity Monitor in OS)"

> "Ideally this activity monitor could run even when it's not being displayed large on the screen, so I can maximize/minimize it."

> "TLDR trying to help dev understand what the agents are doing, what is the bottleneck they're on, if any. Where can concurrency be added, is another big question related to and based on this info."

> "im building out this claude skill as an sdlc-* skill for otel, the example is going to be /sdlc-spike mcp-run-telemetry — track all agent/subagent activity per run in a local embedded database; start with https://github.com/TechNickAI/claude_telemetry but find the best option"

**Open questions:**
- What data is actually available from the Claude API for sub-agent activity? (OTEL spike answers this)
- Is the Activity Monitor a UI panel in the SDLC web UI, or a separate window?
- How do we translate token cost ($) into meaningful quota terms (% of daily limit, estimated remaining)?
- What's the minimum viable version — just "running vs. waiting" per agent, or full breakdown by type?
- How does this connect to the run history / RunRecord model already in the server?
