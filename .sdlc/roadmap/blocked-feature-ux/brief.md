# Blocked Feature UX — Surfacing Options When a Feature Is Blocked

**Origin:** Extracted from Discord conversation dump (sdlc early-user feedback session)

**Summary:**
When a feature is blocked, the current UI shows a "blocked" state but the Run button does nothing useful — it just keeps repeating the blocker message. Xist had no way to: (a) see what the dependency is, (b) navigate to the dependency, (c) waive the blocker, or (d) give the agent a different resolution strategy. He worked around this by using Claude Code directly (`/sdlc-run product-content`). The agent itself knows what to do when unblocked — the gap is the UI not surfacing options, and not detecting when the blocker is resolved externally.

**Key signals (all strong):**
- [Product/User] "whenever I went to click the Run button on the blocked item, it never tried to actually run. It just kept running and telling me 'I'm blocked'" — Run button is a no-op when blocked
- [Product/User] "Ideally there is a button 'go to the dependency so you can click that button'" — cross-feature navigation
- [Product/User] "I just needed a way to enter a text command to give it a new way to resolve the blocker that's different from how it wanted to resolve it." — user wants to override the resolution strategy
- [Engineering] "I planned 2 projects at the same time, and cross-referenced them. So that's probably what caused it." — cross-project dependencies are a real case
- [Engineering] "The blocker was already resolved, the UI didn't know it tho." — overlaps with SSE state reliability but the UX issue is distinct: no refresh action available

**Relevant excerpts (verbatim):**
> "The problem with the UI is whenever I went to click the Run button on the blocked item, it never tried to actually run. It just kept running and telling me 'I'm blocked'. I thought by clicking that it would try to clear the blocker, but it didn't."

> "Ideally there is a button 'go to the dependency so you can click that button'"

> "TLDR on that issue, I just needed a way to enter a text command to give it a new way to resolve the blocker that's different from how it wanted to resolve it."

> "i should surface options, right?" — Jordan agreeing the UI should show choices (waive, go to dependency, check status)

**Open questions:**
- What options should be surfaced on a blocked feature? (waive, navigate to dependency, check status, override with custom instruction)
- Can the blocker detection be made smarter — polling the dependency's state and auto-resolving when it completes?
- How does cross-project blocking work in the data model? Is that supported at all?
- Should "unblock with instruction" be a text input that gets passed directly to the agent?
- What is the relationship between this and the SSE state reliability ponder — do they share a fix?
