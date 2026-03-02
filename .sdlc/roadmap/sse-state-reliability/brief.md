# SSE State Reliability — Real-Time State Sync Reliability

**Origin:** Extracted from Discord conversation dump (sdlc early-user feedback session)

**Summary:**
Three distinct UI state bugs with the same root cause: the UI doesn't reliably re-sync with server state after certain interactions. After collapsing and expanding a ponder panel, it shows "spawning agent" again even though the agent was already running. After UAT finishes, the button resets instead of advancing to the next state. After a blocker is resolved externally, the Run button still shows the item as blocked. All three erode trust — the user can't tell if the system is working or stuck. Jordan has already fixed the UAT issue locally. The pattern suggests SSE/state sync isn't authoritative on component re-mount.

**Key signals (all strong):**
- [Engineering] "I collapsed it, expanded it again not long thereafter, now it says again it's spawning agent, and doesn't show me the status anymore" — state regression on re-mount
- [Product/User] "The 'Run UAT' button finished, and it showed this [completion]. But after I reload the page, it shows this (what it should have showed before)" — UAT state not persisted to UI correctly
- [Product/User] "The blocker was already resolved, the UI didn't know it tho. It did then actually go do stuff." — stale blocked state after external resolution
- [Engineering] Jordan: "Oh yeah, i have seen that, sometimes it gets confused that it needs to update the state and then I go tweak the skill" — confirms it's a known issue

**Relevant excerpts (verbatim):**
> "On the 3rd one, (new one) the first time I expanded it, it said it had spawned the agent and it was working. I collapsed it, expanded it again not long thereafter, now it says again it's spawning agent, and doesn't show me the status anymore"

> "The 'Run UAT' button finished, and it showed this [Image] But after I reload the page, it shows this (what it should have showed before): [Image] So like that 'Run UAT' button doesn't update correctly when it ends, I guess. It just resets itself so you can click it, but that's not what you want, you actually want to move to the next thing, whatever that is."

> "The blocker was already resolved, the UI didn't know it tho. It did then actually go do stuff."

> "Oh I fixed that uat issue locally. I'll be sure to push tonight"

**Open questions:**
- Is the root cause that SSE events are missed on reconnect/remount, or that state isn't pushed after certain transitions?
- Should the UI poll as a fallback when SSE is stale (or just hit the REST endpoint on expand)?
- For the "spawning agent" regression: is state stored in component memory or in a server-side record?
- What's the authoritative source of truth for "is this agent currently running" — server RunRecord or SSE stream?
- Is the UAT local fix already the right fix for all three variants, or are they separate issues?
