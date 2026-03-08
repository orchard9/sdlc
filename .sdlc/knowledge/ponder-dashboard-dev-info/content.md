---
session: 1
timestamp: 2026-03-08T00:15:00Z
orientation:
  current: "Idea is well-scoped — infrastructure already exists, design is tiered, user attribution is the only backend change"
  next: "Commit to milestones — Tier 1 is a single feature, Tier 2 is a follow-up, Tier 3 is a separate feature"
  commit: "Met — the design is concrete, the data model is understood, and the effort is clearly bounded"
---

**Xist · Owner**
dashboard dev info

on the dashboard ui, show whether a milestone or feature is currently running or if it is not.

if it is possible to show which user is currently working on the feature and/or milestone, show that too.

---

**Facilitator**
Good brief. Let me ground this against what we actually have in the codebase before we design anything. I pulled the data model and UI structure — here's what I found:

The backend already has `RunRecord` with `key`, `run_type`, `target`, `status`, `started_at`. The frontend already has `AgentRunContext` with `isRunning(key)` and `getRunForKey(key)`. SSE pushes `RunStarted` and `RunFinished` events. `MilestoneDigestRow` already checks `isRunning(nextFeature.slug)` — but only for the single next feature, not all features in the milestone.

So the plumbing is 90% there. The question is: what exactly should the UI show, and where?

Let me bring in Ben and Dana.

---

**Ben Hartley · Developer UX Designer**
This is a classic information density problem. The dashboard already has a lot going on — phase badges, progress bars, task counts, blocked indicators, UAT screenshots. Adding another indicator needs to earn its space.

My recommendation: a small pulsing dot. Green when running, absent when not. That's it for the default view. Developers scan dashboards — they don't read them. A pulsing animation is the fastest signal that "something is alive here."

For milestones, aggregate it: "2 running" as a small chip next to the progress bar. Don't list which features — the milestone row already expands to show features, and each feature card would have its own dot.

Hover or click for details — run type, duration, turn count. This is progressive disclosure done right.

---

**Dana Cho · Product Skeptic**
I buy the running indicator. It answers the question every developer has when they look at a dashboard: "is anything happening right now, or is this stalled?" That's real value.

But I want to push back on user attribution. Who is this for? In local mode, there's one user — you already know who's running things. In cluster mode with project pods, each pod is typically one project with one agent. The "who" is almost always "the agent" or "me."

The only scenario where "who" matters is a shared project pod with multiple humans kicking off runs. Is that a real scenario today?

---

**Facilitator**
Dana raises a good point. Let me check — in the current fleet architecture, each project gets its own pod. Multiple users could access the same project pod via the web UI, but in practice it's usually one team per project. The auth context (Google OAuth) does know who the user is in cluster mode, so the data *could* be captured cheaply.

---

**Ben Hartley**
I'd still add `created_by` to RunRecord even if we don't display it prominently yet. It's one optional string field, backward-compatible, and it becomes useful the moment you have any kind of run history view. "Who started this failed run 3 hours ago?" is a question that will come up.

Display it as small muted text or initials next to the running indicator — only in cluster mode where auth context is available. In local mode, omit it entirely.

---

**Dana Cho**
Fine — capture the data, but don't design the UI around it. The running indicator is the feature. User attribution is a data enrichment that pays off later.

---

**Facilitator**
Alright, let me crystallize this into a tiered design:

**Tier 1 — Running Indicator (zero backend changes)**
- FeatureCard: pulsing green dot next to phase badge when `isRunning(slug)` is true
- MilestoneDigestRow: "N running" chip when any child feature or UAT has an active run
- Data: already available via `AgentRunContext`

**Tier 2 — Run Details Popover (zero backend changes)**
- Hover/click the running indicator to see: run type, started_at (relative), turns, cost
- Data: `getRunForKey()` already returns full RunRecord

**Tier 3 — User Attribution (small backend change)**
- Add `created_by: Option<String>` to RunRecord
- Populate from auth session (cluster) or `git config user.name` (local)
- Display as initials/avatar next to indicator in cluster mode

? Open: Should Tier 3 also track the user in the heartbeat payload so the hub dashboard can show "who's active" across the fleet?

⚑ Decided: Tier 1 is the feature. Tier 2 is a fast follow. Tier 3 is a separate feature that enriches the data model.

⚑ Decided: Pulsing dot (not spinner) — spinners imply "loading/waiting," dots imply "alive/active." The dashboard is showing liveness, not progress.

⚑ Decided: Milestone aggregation shows count ("2 running"), not individual feature names — the expanded view handles that.

---

**Ben Hartley**
One more thing — the running indicator should also cover UAT runs and ponder runs, not just feature runs. A milestone with a UAT running is actively being tested. A ponder entry with a session running is actively being explored. Same pulsing dot, same pattern.

---

**Facilitator**
Good call. The `AgentRunContext` already tracks all run types via the `key` field pattern (`sdlc-run:slug`, `uat:slug`, `ponder:slug`). We just need to check all relevant keys per entity.

---

**Summary of decisions:**

1. **Tier 1** is a single feature: add pulsing running indicators to FeatureCard and MilestoneDigestRow. Zero backend changes. Uses existing `AgentRunContext`.
2. **Tier 2** is a follow-up feature: add hover popover with run details. Zero backend changes.
3. **Tier 3** is a separate feature: add `created_by` to RunRecord, populate from auth/git, display in cluster mode.
4. All run types (feature, UAT, ponder) get the same indicator treatment.
5. Milestone aggregation is a count, not a list.

This is ready to commit to milestones.
