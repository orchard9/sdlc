# P4 VCS Integration — P4/Perforce Multi-Developer Support

**Origin:** Extracted from Discord conversation dump (sdlc early-user feedback session)

**Summary:**
Xist's primary codebase (MasqMain) uses Perforce (P4) for version control, not git. P4 makes files readonly by default — checking them out requires an explicit `p4 edit` command. This means `sdlc init` fails with "Permission denied" when it tries to write AGENTS.md (and potentially other files). Xist had to manually run `p4 edit AGENTS.md` to proceed. This is an enterprise game development reality (many studios use P4 for large binary assets alongside code). The P4 thread had 58 messages — significant exploration of the problem. This is a blocker for the enterprise/studio market.

**Key signals (both strong):**
- [Engineering] "Error here, guessing having to do with p4 making things readonly?" — hit the issue on first `sdlc init`
- [Engineering] Thread "P4 + `.sdlc/` <-- multi dev update required" with 58 messages — significant technical exploration happened here
- [Process] "ok i will p4 edit AGENTS.md" — manual workaround required, and it's non-obvious
- [Strategy] "first p4 thing haha, didn't take long" — they expected this to come up eventually; it did on day 1

**Relevant excerpts (verbatim):**
> "Error here, guessing having to do with p4 making things readonly?"

> "error: Permission denied (os error 13) — Doesn't say what file experienced permission denied."

> "ok i will p4 edit AGENTS.md"

> "SDLC updated to v0.1.0." (after manually editing AGENTS.md in P4)

> Thread title: "P4 + `.sdlc/` <-- multi dev update required" — 58 Messages

**Open questions:**
- What does the P4 + .sdlc/ thread conclude? What were the 58 messages about?
- Should sdlc detect P4 and offer to run `p4 edit` before writing files, or should it print better error messages with the specific filename and the p4 command to run?
- How does multi-developer .sdlc/ state work in a P4 depot? (git has branches; P4 has streams/changelists)
- Is the `.sdlc/` directory appropriate to put in P4 at all, or should it live outside the depot?
- What's the scope of impact — just AGENTS.md, or all .sdlc/ writes?
- Are there other VCS systems (SVN, etc.) with the same read-only-by-default problem that we'd want to handle the same way?
