# Signal Watch — Weak Signals Parking Lot

**Origin:** Extracted from Discord conversation dump (sdlc early-user feedback session)

**Summary:**
Signals that appeared once, were mentioned briefly, or don't yet have enough context to form a full ponder. Worth revisiting as more conversations accumulate.

**Weak signals:**

- **Arrow keys in chat input** (Bug): "When typing, clicking left/right arrow keys DO NOT move the cursor in this text box. Instead, arrow keys navigate the miniature collapsed window with illegible text." — single mention, concrete bug, quick fix
- **Column resizing** (UX): Full screen on MacBook Pro, columns on right aren't usable. "Ideally we could maybe resize those columns, and/or collapse columns we dont need at any given moment?" — one thread, layout concern
- **"User Pondering" status** (Feature): 2-message thread — a feature request for a status that indicates the user is actively reviewing/thinking about something. Interesting concept, minimal signal so far
- **Links not working** (Bug): 3-message thread, title "Bug: Links not working." No details in main channel — needs thread content to evaluate
- **Realtime work updates** (Feature): 5-message thread. "Feature Request: Realtime work updates" — more detail needed from thread
- **gitea → woodpecker → zot CI/CD stack** (Strategy): Jordan mentioned wanting to fork/rewrite these for agent use. "controllable ci/cd" as strategic direction for dev-box workflows. Too early-stage to ponder now
- **How to get updates** (Process): "When you make changes, how do I get the changed?" — `sdlc update` exists, needs documentation. May be resolved by install-onboarding-ux ponder
- **MCP/Agent no recovery from 500s** (Engineering): "The MCP/Agents never recover" from Claude 500 errors. Mentioned once, but potentially high severity. Needs more investigation
- **Session resume** (UX): "Is there a way to get the 2nd one to resume, or restart or something?" — ponder run resumed after sleep. Mentioned once but an interesting capability gap

**Relevant excerpts (verbatim):**
> "When typing, clicking left/right arrow keys DO NOT move the cursor in this text box. Instead, arrow keys navigate the miniature collapsed window with illegible text."

> "Ideally we could maybe resize those columns, and/or collapse columns we dont need at any given moment?"

> "The agents/tasks keep stopping? I've noticed that when claude is acting up I don't handle it. I noticed that today, like when claude issues a 500 or whatever. The MCP/Agents never recover"

> "Is there a way to get the 2nd one to resume, or restart or something?"

**Open questions:**
- Which of these grows into its own ponder as more signal accumulates?
- Is "Links not working" a quick fix or a deeper routing/navigation issue?
- Is MCP 500-recovery important enough to pull out of signal-watch into its own ponder now?
