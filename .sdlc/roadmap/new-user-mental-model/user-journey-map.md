# User Journey Map: New User First 30 Minutes

Mapping what a new user **currently experiences** step by step vs. what they **ideally should experience** in their first 30 minutes with SDLC.

---

## Current Journey (What Xist Actually Experienced)

### Minute 0–2: Installation

**Action:** `cargo install sdlc` or clone + build. User opens terminal and types `sdlc`.

**What they see:** CLI help output. No story. No "start here."

**What they understand:** "This is a CLI tool. I should probably open the server." They open the server and navigate to `localhost`.

**Failure mode:** No mental framing before the UI loads. The user arrives at the UI without knowing what the tool's core loop is.

---

### Minute 2–5: First Dashboard View

**Action:** User opens the browser. Dashboard loads.

**What they see:** An amber banner: "Project setup is incomplete — agents won't have enough context to work with." A link: "Go to Setup →"

**What they understand:** "I broke something. I need to set up Vision and Architecture before I can use this." They feel behind, not welcomed.

**Critical failure:** The dashboard's first communication is a warning, not an invitation. The user's first emotion is "I did something wrong" rather than "here's where to start."

**What's missing:** Any explanation of *what the tool does at all*. A user who has never heard of SDLC has no idea what "Vision" and "Architecture" mean in this context, or why agents need them.

---

### Minute 5–15: The Setup Wall

**Action:** User clicks "Go to Setup →". They land on the Setup page.

**What they see:** A four-step wizard: (1) Project Description, (2) Vision, (3) Architecture, (4) Team. Each step has a text area.

**What they understand:** "I have to write a Vision document before I can use the product." This feels like homework. They have no idea what an agent will do with their Vision once they write it.

**Divergence point:** Two types of users diverge here.
- *Xist-type:* Writes placeholder text, skips ahead, wants to explore before committing. Gets through setup but feels like they cheated.
- *Other users:* Stops here. Goes and reads docs. Or quits.

**What's missing:** A "quick start" path that says "Write two sentences about your project — that's enough to try the tool." And crucially: a preview of what will happen *after* setup. "Once you complete this, agents will build your software for you" is not said anywhere.

---

### Minute 15–20: Back to the Dashboard, Still Confused

**Action:** User completes setup (or skips some steps) and lands back on Dashboard.

**What they see:** A project overview. Feature count: 0. Milestone count: 0. The "active" and "done" stats are both empty.

**What they understand:** Nothing. The dashboard has nothing to show. There's no "start here" prompt, no suggested action, no explanation of what to do next.

**Critical failure:** Empty state with no guidance. User stares at an empty dashboard and doesn't know if they should create a feature, create a milestone, open the Ponder page, or run a CLI command.

---

### Minute 20–28: Creating Individual Features

**Action:** User navigates to Features. Creates a feature. Then another. Then more. Creates 20+ features.

**What they understand:** "This is a task list. I create tasks and run them one at a time." The UI affordance of a feature list with individual "Run" buttons reinforces this.

**Critical failure:** The user has learned completely the wrong mental model. They are now operating SDLC as if it's a fancy Jira with AI. The product's most powerful feature — run-wave, autonomous parallel execution — is invisible.

---

### Minute 28–30: Discovery (by accident or by asking Jordan)

**Action:** User either discovers Milestones page and sees "Run Wave" button, or asks someone.

**What happens when they finally see Run Wave:** "Oh — *this* is what the tool does. Why did I spend 20 minutes doing it the other way?"

**Current aha moment:** Happens by accident or by word-of-mouth. Not designed. Not reliable.

---

## Ideal Journey (Designed for the Correct Mental Model)

### Minute 0–2: First Contact

**Action:** User opens the server for the first time.

**What they see:** Not a warning banner. Instead: a single-sentence description of the tool's core loop. Something like: "SDLC turns ideas into shipped software. You think in ideas — agents think in waves of parallel work."

**Below that:** Two paths.
- "I have a project in mind" → goes to a streamlined Ponder entry creation
- "Show me how it works" → shows a 60-second animated explainer of the ponder → plan → run wave flow

**Critical change:** The tool communicates its identity before asking the user to do setup.

---

### Minute 2–8: The First Ponder (Not Setup)

**Action:** User is guided to create a Ponder entry — not to write a Vision document.

**What they see:** A text field: "Describe what you're building in one to three sentences." That's it. No "Vision" or "Architecture" labels. No four-step wizard.

**What happens:** The tool uses that description to pre-fill a draft Vision and Architecture. The user sees: "Here's what I understood. Is this roughly right?" They say yes. Setup is done in 90 seconds.

**Why this order matters:** The user's first action is creative and low-stakes (describe an idea) rather than administrative and high-stakes (write formal Vision/Architecture docs). They're already *in the tool* before they've had to think about process.

---

### Minute 8–12: The Flow Reveal

**Action:** After the first ponder is created, the tool surfaces the flow explicitly.

**What they see:** A visual pipeline at the top of the screen — not a sidebar, not a tooltip. A horizontal flow: `Ponder → Plan → Commit → Run Wave → Ship`. Each stage is labeled. The current stage ("Ponder") is highlighted.

**Below the pipeline:** The tool says: "Your first ponder is ready. When you're satisfied with the idea, `/sdlc-ponder-commit` will convert it into a milestone with waves of parallel agent tasks. Then you click Run Wave and agents build it."

**Critical change:** The user now knows where they are in a flow, and they can see where they're going.

---

### Minute 12–20: The First Run Wave (or Meaningful Progress Toward It)

**Action:** User commits the ponder. A milestone is created. They see the Wave Plan.

**What they see:** Wave 1 with N features, each with a "Run" button. And above the wave: a big "Run Wave" button that runs all of them simultaneously.

**What they understand:** "I click this and all N agents start working in parallel. I can go do something else."

**First aha moment is designed, not accidental.** The product has led them to this moment deliberately.

---

### Minute 20–30: Fire and Check In

**Action:** User clicks Run Wave. Agents start. User sees activity in the live log panel.

**Two modes served:**
- *Xist-mode:* Watch the log, understand what each agent is doing, build trust through observation. The live log is designed for this: clear, readable, each agent's actions legible.
- *Jordan-mode:* Click Run Wave, minimize the window, come back in 30 minutes. The tool doesn't require watching. Notifications appear when waves complete.

**End of first 30 minutes:** User has a running wave. They understand the core loop. They are using the tool correctly.

---

## Gap Summary

| Moment | Current Experience | Ideal Experience |
|---|---|---|
| First screen | Warning: "setup incomplete" | Invitation: tool identity + two paths |
| First action | Fill in Vision/Architecture | Create a Ponder in one paragraph |
| Setup framing | Homework before the real thing | Rapid pre-fill with confirmation |
| Flow visibility | Invisible | Persistent pipeline indicator |
| Run Wave discovery | Accidental / word-of-mouth | First deliberate milestone action |
| Fire-and-forget | Not communicated | Explicit — "go do something else" |
| Aha moment | 20–60 minutes, if ever | Designed to occur at minute 12–15 |
