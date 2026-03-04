---
session: 1
timestamp: 2026-03-03T09:35:00Z
orientation:
  current: "Two distinct surfaces show 'Run UAT' — MilestonePreparePanel (milestone-level) and FeatureDetail (feature run_qa action). Jordan wants a human path for both. Design is clear: secondary button alongside existing AI button, modal with checklist from qa_plan/acceptance_test.md, verdict selection, POST to new REST endpoint."
  next: "Create feature: write spec for human-run-uat covering both surfaces, data model changes (UatRun runner field), REST endpoints, and UI modal design."
  commit: "Design shaped — ready to create feature and write spec."
---

**jordan · Owner**
human run uat

whenever we show "run uat" i want a way for a human to complete uat by providing their results

---

## Session 1 — Scoping Human-Run UAT

**Facilitator**
Jordan's request is deceptively simple: "when we show run uat, give the human a way to submit results." But there are two distinct surfaces in the codebase where "Run UAT" appears, and they have different data models. Before designing anything, we need to understand both — and whether we're solving both or one.

I've mapped the codebase. Here's what I found:

**Surface 1 — Milestone UAT (`MilestonePreparePanel.tsx:73`)**
- Shown when all features in a milestone are released (the `VerifyingMini` component)
- "Run UAT" button starts an agent run via `POST /api/milestone/{slug}/uat`
- Creates a `UatRun` record in the milestone with `verdict: pass | pass_with_tasks | failed`, `tests_total`, `tests_passed`, `tests_failed`, Playwright report path
- Results written to `.sdlc/milestones/{slug}/uat-runs/{date}-{id}/summary.md` and `uat_results.md`

**Surface 2 — Feature-level QA (`FeatureDetail.tsx`, action `run_qa`)**
- Shown when a feature is in QA phase and `qa_results` artifact is missing
- "Run" button starts an agent run directive
- Agent writes `.sdlc/features/{slug}/qa-results.md` as a Draft artifact
- Then `approve_merge` directive fires for the agent to verify and approve

---

**Dan Reeves (Systems Minimalist):** Before we build anything, let me ask the obvious question. The human result is just a markdown file. For feature QA, can the human not just open `.sdlc/features/{slug}/qa-results.md`, write their results, and run `sdlc artifact draft {slug} qa_results`? That's the whole loop — no new API endpoint, no new UI.

**Facilitator:** Dan, that's a valid CLI path but it requires knowing the file path, the artifact command syntax, and comfort with the terminal. The web UI doesn't expose that workflow at all. Jordan is running this as a web-first product — the `Run UAT` button is the primary surface. The human path needs to be a peer of the AI path in the UI, not a terminal workaround.

**Dan:** Fine. But whatever we build, the data model impact should be zero. Milestone UAT already has `UatRun`. Feature QA already has an artifact lifecycle. Don't create new primitives if you can fit human results into the existing shapes.

⚑ **Decided:** Reuse existing data models. Feature QA writes qa-results.md as a Draft artifact (same as agent path). Milestone UAT creates a UatRun record (same as agent path) with an added `mode: "human"` field to distinguish the source.

---

**Dana Cho (Product Skeptic):** Who is the human doing this? Is it Jordan testing his own feature? A QA engineer on a team? A product manager signing off? Because the answer changes what information they need to provide.

**Facilitator:** Given the product context — this is a solo developer tool going to small enterprise teams — the human is most likely the developer or a teammate doing a real-world test that Playwright can't automate. Think: "I clicked through the flow manually on staging, here are my results."

**Dana:** Then the form should be minimal. Don't ask them to re-enter every checklist item. The qa_plan/acceptance_test.md already has the checklist — show it, let them check each item, add notes if they want, pick a verdict. That's it. One screen, one submit. Any more than that and the human just runs the AI instead.

⚑ **Decided:** The form shows the existing checklist (from qa_plan for features, from acceptance_test.md for milestones). Human checks/unchecks items. Overall verdict auto-suggests based on checks but can be overridden. Notes textarea for anything else. Submit creates the result.

---

**Ben Hartley (Dev Productivity UX):** The key cognitive load question: where does the "Submit Human Results" button live? You have two options.

Option A: Replace the "Run UAT" button with a dropdown — "Run with AI" / "Submit Human Results." Good for discoverability, bad for speed (an extra click for the 95% case).

Option B: Keep the existing "Run UAT" button. Add a secondary "Human" button or link beside it, visually subordinate. Good for speed — AI stays one click, human is available without obscuring the primary path.

Given that AI runs are the primary case and human runs are the exception (when Playwright isn't working, or you want manual verification), Option B is correct. The human path should be clearly accessible but not competing for primary position.

⚑ **Decided:** Option B — secondary "Human" button/link alongside existing "Run UAT" button. Label: "Submit manually" or a person icon. Lower visual weight than the primary AI button.

---

**Tobias Krenn (Skeptical Engineering):** How much does this actually change the backend? Let's count the new primitives:

For **feature QA**: Zero backend changes. The human fills a form, the client constructs qa-results.md content from the form data, and POSTs to a new endpoint that writes the artifact as Draft. Or simpler: `POST /api/features/{slug}/artifacts/qa_results` with a body — actually we might already have artifact write endpoints.

For **milestone UAT**: One field addition to `UatRun` — add `mode: Option<String>` (or an enum `UatRunMode::Human | Agent`). The new `POST /api/milestone/{slug}/uat/human` endpoint creates a UatRun with `mode: "human"`, writes the summary, and updates uat_results.md. The existing UatRun struct otherwise handles everything.

That's manageable. Two new REST endpoints, one new field. Don't over-engineer the mode — a string "human" or "agent" in UatRun is fine. Don't create a full enum unless tests demand it.

? **Open:** Does the existing feature artifact write path (`sdlc artifact draft {slug} qa_results`) accept content via stdin/file, so the backend endpoint can just shell out to it? Or does the endpoint need to write the file directly?

**Dan:** The server writes files directly — no shelling out. That's fine. The endpoint writes qa-results.md via the atomic write path in `sdlc-core/src/io.rs`, then calls the artifact status update. Consistent with how agents do it.

⚑ **Decided:** Backend implementation — two new REST endpoints:
1. `POST /api/features/{slug}/human-qa` — writes qa-results.md from form data, sets artifact to Draft
2. `POST /api/milestone/{slug}/uat/human` — creates UatRun with mode="human", writes summary + uat_results.md

---

**Ben Hartley:** One UX detail that matters: when the modal opens and shows the checklist, the human needs to know what "passing" means for each item. The qa_plan is already Markdown — rendering it as-is in the modal is better than parsing it into checkboxes, because the items may have context, sub-steps, and criteria that get lost if you just strip them to bullet points.

Suggested approach:
- Render the qa_plan/acceptance_test.md as read-only Markdown in the upper section of the modal
- Below it, provide a simple verdict selector: `Pass / Pass with Tasks / Fail`
- A textarea for notes
- A "Submit" button

This respects the existing artifact structure rather than trying to parse and re-structure it. The human reads the plan, does the testing, then records the outcome. That's how real QA works.

? **Open:** For feature QA specifically — should the form also accept individual test item results (check per line), or just an overall verdict + notes? And should the resulting qa-results.md be the form data formatted into markdown, or a freeform "what the human writes"?

**Dana:** Don't over-structure it. The value is in getting the verdict recorded and unblocking the state machine. The human notes field is the evidence. Format qa-results.md as: verdict heading, notes, and a timestamp with "Completed by: Human (manual)." That's enough for approve_merge to fire.

⚑ **Decided:** Form structure: render qa_plan as read-only Markdown reference → verdict selector (3 options) → notes textarea. Generated qa-results.md contains verdict, timestamp, notes, runner="human". No per-item checkbox parsing.

---

**Tobias Krenn:** One more risk to name: after the human submits, what happens? For feature QA, the artifact goes to Draft, and the `approve_merge` directive fires — which currently tells an agent to verify and approve. Is there an intent to have the human also approve, or does the agent still do the approval step?

**Facilitator:** Jordan's request was specifically about completing the UAT by providing results. The approval step (which currently means "verify the results are real and merge-worthy") can remain agent-driven. The human provides the evidence; the agent approves based on that evidence. This is the minimal change.

? **Open:** Should "human-submitted" results auto-approve (skip the agent approval step), or still require an agent review? Auto-approve is simpler but loses the double-check. Agent review is more principled but adds latency.

**Dan:** Leave the approval step agent-driven. The human's job is to record the results. The agent's job is to verify the artifact is complete and approve. Don't conflate them — you'll want that separation when someone submits a "Pass" with no evidence.

⚑ **Decided:** Human submission writes Draft artifact. Approval step remains agent-driven (same as always). No change to the approval flow.

---

## Design Summary

**Where it appears:**
1. `MilestonePreparePanel.tsx` — "Run UAT" button → add secondary "Submit manually" link
2. `FeatureDetail.tsx` — `run_qa` action panel → add secondary "Submit manually" button

**What the human provides:**
- Reference: qa_plan.md (feature) or acceptance_test.md (milestone) rendered read-only
- Verdict: Pass / Pass with Tasks / Fail
- Notes: freeform textarea (required for non-Pass)

**What gets written:**
- Feature: `qa-results.md` as Draft artifact — standard format with `## Verdict`, `## Notes`, `Runner: human (manual)`, timestamp
- Milestone: `UatRun` record with `mode: "human"`, standard `verdict`, `tests_total/passed/failed` from form, `summary.md`

**Backend changes:**
- New field: `UatRun.mode: Option<String>` (values: `"agent"` | `"human"`)
- New endpoint: `POST /api/features/{slug}/human-qa` → writes qa-results.md, sets artifact Draft
- New endpoint: `POST /api/milestone/{slug}/uat/human` → creates UatRun(mode=human), updates uat_results.md

**What stays the same:**
- Approval flow unchanged — agent still approves after human submits
- CLI path unchanged — humans can still write files directly
- AI "Run UAT" button unchanged — human path is additive, not a replacement

---

## Commit Signal Met

The design is shaped and minimal. Two surfaces, two endpoints, one new field, one UI modal reused across both surfaces. The feature is ready to spec.
