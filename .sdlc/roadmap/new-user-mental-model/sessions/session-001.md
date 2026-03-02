---
session: 1
timestamp: 2026-03-01T00:00:00Z
orientation:
  current: "Jordan and Xist have completely different mental models of the tool — Jordan's flow is never surfaced"
  next: "Map the gap: what does a new user understand after 1 hour with the tool vs. what they need to understand to use it effectively?"
  commit: "Concrete proposal for how to surface Jordan's ponder→plan→commit→run-wave flow to new users, without requiring Jordan to explain it"
---

## Session 1: Initial Signal Load

Bootstrapped by /sdlc-convo-mine from Discord conversation (sdlc early-user feedback session).

### Signals extracted

- **Jordan's flow is invisible**: ponder → plan → converge → commit → run wave. This flow is not shown anywhere in the UI. New users discover it by asking Jordan.
- **Run Wave hidden behind setup**: Run Wave doesn't appear until Vision + Architecture are set up. The dependency is correct but unexplained — users see a wall, not a path.
- **Xist started 20+ features individually**: Because he didn't know about Run Wave. The dashboard let him do this without any guidance.
- **"I do not know how to use this tool"**: Said twice. Clear sign that the mental model gap is wide.
- **"I just want it to build everything then I'll look at the final result"**: This is actually the correct SDLC use case. Xist's instinct is right. The tool just doesn't communicate that this is achievable.
- **Watching vs. fire-and-forget dichotomy**: Jordan never watches running agents; Xist always watches until he understands. The UI needs to serve both modes.

### Why this might matter

The mental model gap is the #1 adoption barrier. If users understand ponder → run wave in the first 5 minutes, they use the tool very differently. If they don't, they use it as a complicated to-do list and get frustrated. This is a product design problem, not just a docs problem.

### Open questions

- What is the minimum intervention that teaches the flow? An onboarding walkthrough? An empty-state prompt? A "suggested next" button?
- Should the dashboard show the flow as a visible pipeline (ponder → plan → wave running → results)?
- How do we teach "fire and check in later" without undermining users who want to watch?
- Is the Vision/Architecture setup the right gate, or should new users be able to start a ponder immediately?
- What would a "1-hour new user experience" look like if we designed it intentionally?

### Suggested first exploration

Run an empathy session (/sdlc-empathy) specifically focused on the first-hour experience. Model two users: (1) Xist-style — enterprise dev, Agy background, watchful, (2) Jordan-style — fire-and-forget, CLI-comfortable, iterative. Design for Xist; Jordan will be fine.
