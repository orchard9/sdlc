# Install Onboarding UX — Installation and First-Run Experience

**Origin:** Extracted from Discord conversation dump (sdlc early-user feedback session)

**Summary:**
Xist's first session with SDLC was dominated by installation friction. The `cargo install --git https://` URL only works for users with a single SSH key — a problem for engineers with multiple keys (common on enterprise machines). The `error: Permission denied (os error 13)` provided no filename and no debugging path. `make install` exists as the easy path but isn't mentioned in install instructions. Once installed, the UI didn't clearly explain what Vision and Architecture are or how to create them, leaving Xist guessing. DEVELOPER.md was written but never communicated. The onboarding experience is entirely word-of-mouth right now.

**Key signals (all strong):**
- [Process] "All the install instructions don't work for me. I cannot use https://github.com/... urls, that only works for people with 1 and only 1 ssh key." — major install blocker for multi-key devs
- [Process] "It would be nice if there was an easy way to install it once I had already checked it out." — `make install` exists but undiscoverable
- [Engineering] "error: Permission denied (os error 13) — Doesn't say what file experienced permission denied. I tried --debug and --verbose switches but no dice on getting more error info." — error messages need filenames
- [Product/User] "What do I do for the Vision and Architecture? Guidelines for making those?" — first-run guidance gap
- [Process] "$ make install is hopefully the easy way to install it, sorry about that, i wrote developer.md for you and forgot to say anything about it" — DEVELOPER.md exists but not communicated

**Relevant excerpts (verbatim):**
> "Took me a bit to get it installed. All the install instructions don't work for me. I cannot use https://github.com/... urls, that only works for people with 1 and only 1 ssh key. Luckily the 'search all my keys' function worked for this install URL: cargo install --git ssh://git@github.com/orchard9/sdlc sdlc-cli"

> "error: Permission denied (os error 13) — Doesn't way what file experienced permission denied. I tried --debug and --verbose switches but no dice on getting more error info."

> "Yeah filenames for errors like that would be good"

> "What do I do for the Vision and Architecture? Guidelines for making those?"

> "$ make install is hopefully the easy way to install it, sorry about that, i wrote developer.md for you and forgot to say anything about it"

> "When you make changes, how do I get the changed?"

**Open questions:**
- What is the intended first-run flow for a brand new user who isn't Jordan?
- Should the UI detect missing Vision/Architecture and prompt with a guided form instead of just showing an empty dashboard?
- How should `sdlc init` errors surface the specific filename that caused the permission issue?
- Is `make install` the right primary install path? Should the README lead with it?
- What does a good "update instructions" story look like — `sdlc update` + some curl or `cargo install`?
