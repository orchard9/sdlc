---
session: 2
timestamp: 2026-03-01T00:00:00Z
orientation:
  current: "Diagnosed the mental model gap as a product design problem, not a UI or docs problem. Proposed concrete interventions ordered by impact."
  next: "Implement A3 (empty state redesign) and C1/C2 (rename + subtitles) as immediate low-effort fixes. Then design the Ponder-first entry path (A1) and pipeline indicator (A2)."
  commit: "Ponder is ready to commit. The design direction is clear: make ponder the first action, make the flow visible, surface Run Wave on the dashboard. Three features + one pipeline component."
---

## Session 2: Mapping the Gap — From New User Confusion to Designed Aha Moment

**Participants:** Maya Goldberg (Onboarding & First-Run Experience Designer), Xist (First-Time User Persona), Priya Chakraborty (Product Strategist & Skeptic)

---

### Opening: What Are We Actually Fixing?

We started by loading context from Session 1 and reviewing the current UI structure.

Key structural findings from the codebase:
- The Dashboard detects setup incompleteness and shows a warning banner before anything else
- Run Wave lives in `WavePlan.tsx`, which is only rendered inside `MilestoneDetail` — four navigation levels deep from the empty dashboard
- The Ponder page has an "empty" state with "Suggest an idea" and "New idea" — this is the closest thing to a "start here" prompt, but it's not shown on first run
- The Setup page is a four-step wizard (Description → Vision → Architecture → Team), all required before the dashboard feels functional
- No persistent flow indicator exists anywhere in the UI

---

### Thread 1: What Kind of Problem Is This?

**Priya Chakraborty · Product Strategist & Skeptic**

Before anyone proposes a solution, I need to name what kind of problem this is. There are three categories:

1. **UI problem**: User can't find something. Fix: move or label it.
2. **Conceptual problem**: User doesn't understand a term. Fix: better labels, tooltips.
3. **Product design problem**: The product's first experience affords the wrong behavior, and no amount of better labels will fix it.

I'm going to argue this is category 3 — and it matters because category 3 solutions are different in kind, not just degree.

Here is the evidence. Xist came in with a clear mental model: *I have a plan, I want AI to help me execute tasks*. That mental model is reasonable and common. The dashboard's empty state says: create features. The feature detail page says: run this feature. The individual "Run" button says: I am a manual action you take one at a time. Every affordance in the UI taught Xist the wrong mental model, not because the labels were wrong, but because the affordances were wrong.

The fix is not a tooltip on the "Run" button. The fix is changing what the product does on first run — specifically, making ponder the first action, not feature creation and not setup.

**Maya Goldberg · Onboarding & First-Run Experience Designer**

Agreed on the diagnosis. At Linear, we had a version of this exact problem early on. The product let you create issues immediately — which is technically correct — but users who created issues before they'd created a project were confused, because issues without projects don't have context. The fix wasn't a warning. The fix was: on first run, make project creation the first action. Once you've created a project, issues make sense.

SDLC has the same structure. Features without a milestone are like issues without a project. They're technically valid but they teach the wrong mental model. The first-run experience should create a ponder, then commit it to a milestone, then show Run Wave — in that order, in that session.

**Xist · First-Time User Persona**

I want to be concrete about what I saw and what I understood. When I opened the dashboard for the first time:

First sentence I read: "Project setup is incomplete — agents won't have enough context to work with."

My interpretation: I broke something. I need to fix it before I can do anything useful. I clicked "Go to Setup."

The setup page asked me to write a Vision. I didn't know what that meant in this context. Is it a product vision document? A mission statement? I wrote a few sentences and moved on.

Then I came back to the dashboard. It was empty. I didn't know what to do. I went to Features because that seemed like the thing you do — create work items. I created features. I ran them one at a time.

What I needed to see in the first 30 seconds was: "This tool runs agents autonomously in parallel. Here's the flow: describe an idea, let us plan it, then click one button and agents build it." That's it. One paragraph. I never saw that paragraph.

?  Open: Is the right place for this paragraph the very first screen, before any setup? Or does setup have to come first for technical reasons?

**Maya Goldberg · Onboarding & First-Run Experience Designer**

Setup does not have to come first. The Ponder page can collect a one-paragraph description that seeds Vision and Architecture in the background. The user doesn't need to know they're doing "setup" — they're just describing their idea. The tool does the rest. This is the standard UX pattern for modern developer tools: extract necessary metadata from the first creative action, rather than asking users to fill in metadata forms.

⚑  Decided: Setup should not be the first action for new users. Ponder should be. The Vision/Architecture forms are power-user controls, not first-run requirements.

---

### Thread 2: The Run Wave Discovery Problem

**Maya Goldberg · Onboarding & First-Run Experience Designer**

Run Wave is the aha moment. Everything before it is setup. The question is: how long does setup take, and how clearly does the path lead to Run Wave?

Current answer: setup is indeterminate (could be 5 minutes, could be 45), and the path to Run Wave is: Setup → Dashboard → Milestones → Milestone Detail → Wave Plan → Run Wave button. That's four navigations after setup. And the Wave Plan only exists if the milestone has been "prepared" (another agent run). So the actual path is: Setup → Dashboard → Create/Import Features → Milestones → Create Milestone → Prepare Milestone → Wave Plan → Run Wave.

That's seven steps before the aha moment. No user will follow that path without a guide.

**Priya Chakraborty · Product Strategist & Skeptic**

Seven steps is a funnel that will lose 80% of users before the aha moment. This is a critical product issue, not an onboarding issue. The number of steps to first-value must be reduced at the product level.

The fastest path to Run Wave should be: install → describe idea → one click. Three steps. Everything in between should happen automatically or in the background.

Is "one click" achievable? What prevents it?

**Maya Goldberg · Onboarding & First-Run Experience Designer**

The blockers to one-click are:
1. Vision and Architecture need to exist for agents to have context (but we can generate drafts)
2. A ponder needs to be created and committed into a milestone
3. The milestone needs to be "prepared" (wave plan generated)
4. The wave plan needs to exist before Run Wave appears

None of these are fundamental blockers. They're all automatable. The tool could:
1. Take a description from the user
2. Generate draft Vision/Architecture
3. Create a ponder and immediately commit it
4. Run prepare in the background
5. Show "Wave ready — click to run" when preparation is complete

Total user actions: write a description, click Run. Two steps.

⚑  Decided: The first-run experience should be designed as a "fast path" to Run Wave. The target is: two user actions (describe + run) before the aha moment.

**Xist · First-Time User Persona**

I would have stayed with this tool if I'd seen Run Wave in the first ten minutes. Instead I spent 30 minutes doing things that felt wrong, figured out it wasn't working, and asked Jordan. "Just use Run Wave" felt like being told there was an obvious shortcut I should have known about. The discovery should be designed, not social.

?  Open: Should the fast path be opt-in ("try the quick setup") or opt-out (the default for new projects)?

**Priya Chakraborty · Product Strategist & Skeptic**

For new projects it should be the default. Power users who want fine-grained control over Vision/Architecture can access those pages at any time — they don't disappear. The default should serve the modal new user, which is Xist, not the power user, which is Jordan.

⚑  Decided: The fast path is the default. Vision/Architecture pages become secondary (editable post-hoc), not prerequisite.

---

### Thread 3: The Watching vs. Fire-and-Forget Dichotomy

**Xist · First-Time User Persona**

I watched every agent run until I understood what it was doing. That's how I work with new tools — I don't trust automation I can't inspect. Jordan, by contrast, fires and forgets. He said he doesn't watch anymore because he's built up trust over hundreds of runs.

The question is: does the UI support my behavior (watch and learn) while also supporting Jordan's (fire and forget)? Or does it implicitly pressure users to do one or the other?

**Maya Goldberg · Onboarding & First-Run Experience Designer**

This is a well-understood UX tension in automation products. The solution is not to pick one mode — it's to make both modes explicit and equally valid.

Current state: the live log exists, so watching is possible. But there's no indication that *not watching* is also valid. A new user sees a wave running with a live log and assumes they should be watching. There's no "you can close this tab" message.

The fix is a single line of UI copy during a running wave: "Agents don't need you here. Results appear when they're done." That single line tells Xist-types: "your instinct to watch is fine." And it tells Jordan-types: "your behavior is correct and encouraged."

**Priya Chakraborty · Product Strategist & Skeptic**

I'll push back slightly. The two modes are not symmetric. Jordan's mode is the *designed* mode — the tool is built for fire-and-forget. Xist's mode (watching until you trust) is a *transitional* mode for new users. Over time, all Xist-types become Jordan-types.

The UI should honor the transitional mode without designing for it permanently. It should make watching easy while gently communicating that fire-and-forget is available. Not: "here are two permanent modes." Rather: "here's what's happening, and here's how you know you don't need to watch."

⚑  Decided: Add "fire and check in" framing to the wave running state. Not a mode toggle — a contextual message that normalizes not watching, especially for first-time wave runners.

---

### Thread 4: "I Just Want It to Build Everything"

**Xist · First-Time User Persona**

That quote is mine. I said it verbatim. I want to explain what was going on in my head when I said it.

I had just realized the tool wasn't going to run my plan automatically. I had 20+ features, and I was running them one at a time, reading the output, clicking next. It was slow. I wanted to just — hand the whole thing to the system and walk away. But I didn't know if that was possible. Nothing told me it was.

What I needed was one sentence, at the beginning, that said: "SDLC builds software autonomously. You describe what you want, agents build it in parallel waves, you check in on results." That sentence would have changed every decision I made in the first 30 minutes.

**Maya Goldberg · Onboarding & First-Run Experience Designer**

This is the one-sentence problem. Every product needs a one-sentence answer to "what does this do?" The answer has to be concrete (not abstract), capability-focused (what it *does*, not what it *is*), and it has to set the user's expectation about their role.

SDLC's current one-sentence answer (inferred from the UI): "SDLC is a lifecycle management tool for features." That's what you'd guess from the dashboard.

SDLC's correct one-sentence answer: "SDLC turns your ideas into shipped software using waves of parallel agents — you think in ideas, they think in work."

The correct sentence should be visible on the first screen. Not buried in docs. Not in a tooltip. First screen.

**Priya Chakraborty · Product Strategist & Skeptic**

The one-sentence is a product positioning statement as much as it's UX copy. It should come from product leadership (Jordan) and then be implemented everywhere — first screen, readme, CLI help, docs homepage. It's not just a UI fix. It's a brand/identity fix.

⚑  Decided: Agree on a one-sentence identity statement for SDLC and implement it as the first thing new users see. Proposed draft: "Describe what you're building. Agents build it in parallel waves — you check in on results."

---

### Thread 5: The Empty State Problem

**Maya Goldberg · Onboarding & First-Run Experience Designer**

Empty states are the most important screens in any product. They're what users see when they don't know what to do. Bad empty states say "nothing here." Good empty states say "here's what you should do next, and here's why."

SDLC's current empty state (no milestones, no features): amber warning banner + stats bar with zeros + nothing.

The amber warning banner is especially damaging. A new user's first emotion should be curiosity or excitement — "let me explore this." An amber warning as the first thing they see triggers anxiety — "I've done something wrong."

My proposed empty state (no milestones, no features):

---
**SDLC turns ideas into shipped software.**
Agents build in parallel waves — you think in ideas, they think in work.

→ Start with a ponder: describe what you're building in one sentence.

[ New Ponder ]  [ See how it works ]
---

No warning. No stats bar with zeros. One clear action. One fallback for the curious.

**Xist · First-Time User Persona**

That would have completely changed my first session. I would have clicked "New Ponder" immediately. That's what I wanted to do — describe my idea. The setup wall made me feel like I wasn't allowed to describe my idea yet.

**Priya Chakraborty · Product Strategist & Skeptic**

I agree on the framing. I want to add one nuance: the empty state should not show if the user has already created milestones or features. It should be a true first-run state that disappears once the user has created anything. Otherwise it becomes condescending for returning users.

Also: "See how it works" should lead to something real — a demo or animated walkthrough of the ponder→run-wave flow. Not docs. Docs are for when you have a specific question, not for first orientation.

⚑  Decided: Redesign the empty dashboard state. Remove the amber setup warning. Add one-sentence identity, one clear first action (New Ponder), and a "See how it works" option. First-run only.

---

### Thread 6: Is the Vision/Architecture Gate Right?

**Priya Chakraborty · Product Strategist & Skeptic**

I want to examine this directly. Why does the tool require Vision and Architecture before it will work well?

The answer is: agents need project context to write good code. Without understanding what the project is, agents produce generic, possibly incorrect output.

That's a real technical requirement. But it's implemented as a hard gate at the wrong level. The gate shouldn't be: "fill in these forms before we show you anything useful." The gate should be: "we need enough context to do a good job — you can give us context by describing your project."

These are the same underlying requirement implemented very differently. The first feels like bureaucracy. The second feels like collaboration.

**Maya Goldberg · Onboarding & First-Run Experience Designer**

Exactly. And the Ponder-first path I described earlier gives the system enough context without explicitly asking for it. When a user says "I'm building a multiplayer game server in Rust with real-time physics sync," that one sentence gives agents:
- Domain: game development
- Technology: Rust
- Requirement: real-time, multiplayer, physics
- Type of system: server

That's enough to generate a draft Vision ("A Rust game server enabling real-time multiplayer physics") and a draft Architecture ("Event-driven server with physics simulation tick loop"). The user confirms, and the tool has what it needs.

The Vision and Architecture forms still exist — they're where the user refines and expands those drafts. But they're not gates. They're editors.

⚑  Decided: Vision and Architecture become editable documents, not prerequisites. The first-run path generates drafts from the initial ponder description and asks for confirmation, not for users to write from scratch.

---

### Synthesis: What We're Actually Proposing

Having worked through all five threads, the intervention set is clear. Ordered by priority:

**Immediate (low effort, high impact):**
1. Empty state redesign: replace warning banner + zeros with identity statement + "New Ponder" CTA
2. Rename "setup incomplete" → "agents need more context" (word change only)
3. Add subtitles to Vision and Architecture pages explaining why agents use them

**Near-term (medium effort, critical impact):**
4. Ponder-first entry for new projects: description → auto-generate Vision/Architecture → confirm → milestone
5. Active milestones with Run Wave button surfaced on Dashboard (not buried in Milestone Detail)
6. Pipeline visibility indicator: Ponder → Plan → Commit → Run Wave → Ship

**First-wave celebration (low effort, medium impact):**
7. One-time "your first wave is running — you don't need to watch" overlay when a wave first starts

**Recovery path (medium effort, medium impact):**
8. Detection of "many features, no milestone" pattern → offer "Organize into milestone" path

The underlying principle across all interventions: **the tool should communicate its identity and the correct mental model through its affordances, not through documentation**. Every screen a new user sees should make the ponder→run-wave loop more legible. No screen should afford the wrong behavior (e.g., creating individual features without a milestone) without also pointing toward the correct behavior.

---

### Commit Signal Assessment

Session 1's commit signal was: "Concrete proposal for how to surface Jordan's ponder→plan→commit→run-wave flow to new users, without requiring Jordan to explain it."

We have that proposal. It includes:
- A diagnosed problem category (product design, not docs)
- A redesigned first-run experience (ponder-first, not setup-first)
- A pipeline visibility mechanism (horizontal flow indicator)
- A Run Wave surfacing change (on Dashboard, not buried in Milestone Detail)
- A fire-and-forget framing addition (contextual copy during wave runs)
- A prioritized implementation plan

This ponder is ready to commit.

**Next:** `/sdlc-ponder-commit new-user-mental-model`
