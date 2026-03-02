# Skill Command Library — Command and Skill Discoverability

**Origin:** Extracted from Discord conversation dump (sdlc early-user feedback session)

**Summary:**
Xist knows `/sdlc-` exists as a namespace and can list commands, but he doesn't know when or why to use each one. Intent and intended usage are the critical missing piece. He also wants to browse across implementations (Agy vs Claude vs others) to compare how different tools handle the same concept. This is both a documentation problem and a product problem — the UI could surface this as a browseable "command catalog" rather than requiring people to read docs or trial-and-error.

**Key signals (all strong):**
- [Product/User] "We need some way to know for each of these things like 'when/where/why would you use this?' with examples, and then have a place to browse them all" — explicit articulation of the need
- [Product/User] "Intent of skill and intended usage is a really big one. Sometimes it's obvious what the intent and intended usage is, other times not so much." — distinguishes clear vs. unclear commands
- [Product/User] Xist asked what Vision and Architecture were — commands to create them exist but he didn't know when to use them
- [Strategy] "also even look at the code and compare it across different implementations (agy vs claude vs other)" — cross-tool comparison as a learning mechanism

**Relevant excerpts (verbatim):**
> "We need some way to know for each of these things like 'when/where/why would you use this?' with examples, and then have a place to browse them all, including what their intent is, their usage examples, and also even look at the code and compare it across different implementations (agy vs claude vs other)"

> "Intent of skill and intended usage is a really big one. Sometimes it's obvious what the intent and intended usage is, other times not so much."

> "What do I do for the Vision and Architecture? Guidelines for making those?"

> "In Claude if you do /sdlc- you should see all of the available skills/commands" — Jordan's current answer: the tool's autocomplete

**Open questions:**
- Is this primarily a docs/README problem (add "when to use" to each command), or a product problem (in-app command catalog)?
- Should the SDLC web UI have a "Commands" page with browseable entries?
- What does a good "when/where/why" entry look like? Is it a one-liner, a scenario, an example run?
- How do we keep this in sync with the actual command templates without duplicating them?
- Is cross-implementation comparison (Agy vs Claude) a nice-to-have for power users, or core to the experience?
