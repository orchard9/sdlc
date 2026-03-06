---
session: 1
timestamp: 2026-03-04T03:35:00Z
orientation:
  current: "Problem is clear, design is converged. Simple primitive: parked_at timestamp on Milestone."
  next: "Commit to milestone + feature. Implementation is ~2 hours: Rust struct change, CLI commands, UI sections, REST endpoints."
  commit: "Owner confirms the park/unpark semantics and UI treatment are what they want."
---

**jordan · Owner**
park milestones

Need a way to park milestones

---

**Facilitator**

Let me pull in the team. The brief is one sentence but the problem is real — I count 17 non-released milestones right now, and at most 2-3 are being actively worked. Everything else is clutter. Let's figure out the right primitive.

---

**Felix Wagner · Data Model & CLI Semantics**

I see the shape of this immediately. You already have `Skipped` (permanent cancellation via `skipped_at`) and `Released` (permanent completion via `released_at`). Both are terminal states with timestamps. What's missing is a **reversible pause state**.

The pattern is: `parked_at: Option<DateTime<Utc>>`. When set, `compute_status()` returns `Parked`. When cleared, status reverts to whatever it would have been (Active or Verifying). Same serde treatment as `skipped_at` — `#[serde(default, skip_serializing_if = "Option::is_none")]`.

Priority order in `compute_status()`:
```
Skipped > Released > Parked > Verifying > Active
```

CLI:
```
sdlc milestone park <slug>
sdlc milestone unpark <slug>
```

No flags, no reason field. The reason is always "not now." If someone needs to explain *why*, they can add a comment to the milestone or just... not. The parking action is its own explanation.

**? Open:** Should we also be able to park individual features, or just milestones? Features have `archived: bool` but that's different — archived means "removed from scope", not "paused."

---

**Dana Cho · Product Skeptic**

Let me challenge whether this needs to be built at all.

The problem is dashboard noise — too many milestones competing for attention. Could we solve this with just a filter? A "show only milestones I'm working on this week" toggle?

...Actually, no. The noise isn't just UI. When agents run `/sdlc-prepare` or look at the milestone list to decide what to work on next, parked milestones shouldn't be candidates. A filter is a UI band-aid over a state problem. **The state machine needs to know the difference between "active and in-flight" and "active but not right now."**

This is cheap to build. The `parked_at` timestamp approach reuses the exact same pattern as `skipped_at`. No migration needed (it's an additive field with `serde(default)`). The Rust change is maybe 20 lines. The CLI change is two subcommands. The UI change is moving one filter line in `MilestonesPage.tsx`.

⚑ Decided: This is worth building. It's the right primitive at the right cost.

One thing though — the 12 "verifying" milestones are going to need triage. Some of them (citadel-pantheon-integration, telegram-digest-bot) were committed from ponder sessions before anyone was ready to work on them. Once park exists, Jordan should park those. But that's a one-time cleanup, not a feature concern.

---

**Tobias Krenn · Over-engineering Check**

I want to make sure we don't gold-plate this. Felix's proposal is correct and minimal. Here's what I'd push back on if anyone suggests it:

1. **Do NOT add a "park reason" field.** Nobody will read it. Nobody will update it. It's a form field that makes the `park` command less frictionless for zero value.

2. **Do NOT add park/unpark history tracking.** "Was this parked before?" is a question nobody will ever ask. If they do, `git log` answers it.

3. **Do NOT change how features inside a parked milestone behave.** Features don't know they're in a parked milestone. Their classifier rules don't change. An agent running `/sdlc-next <feature-in-parked-milestone>` still works — it just won't happen because nobody will invoke it. The state machine is already correct.

4. **Do NOT add "bulk park" as a first-class operation.** If Jordan wants to park 5 milestones, five `sdlc milestone park <slug>` commands is fine. Shell history exists.

The total scope should be:
- `Parked` variant in `MilestoneStatus`
- `parked_at` field on `Milestone` struct
- `park()` and `unpark()` methods on `Milestone`
- `compute_status()` updated with Parked priority
- Two CLI subcommands
- Two REST endpoints
- UI: three-section milestones page, filter parked from Horizon
- Tests: 3-4 unit tests matching the existing skip/release test patterns

That's it. If the PR has more than ~200 lines of non-test Rust changes, something went wrong.

---

**Felix Wagner**

Tobias is right on all counts. On the open question about feature-level parking: **don't build it.** Features already have `archived: bool`. The lifecycle for features is: work on it or archive it. Parking is a milestone-level concept because milestones represent *commitment* — "we are doing this body of work." Features are atoms of that commitment. You park the commitment, not the atoms.

⚑ Decided: Parking is milestone-only. No feature-level parking.

---

**Facilitator**

Clean convergence. Let me summarize:

### What we're building

**A `Parked` status for milestones** — a reversible "not now" state that removes milestones from active dashboard views, Horizon zone, and agent consideration, without cancelling them.

### Implementation

| Layer | Change |
|-------|--------|
| `milestone.rs` | Add `Parked` to `MilestoneStatus`, add `parked_at: Option<DateTime<Utc>>`, add `park()` / `unpark()` methods, update `compute_status()` |
| `sdlc-cli` | Add `park` and `unpark` subcommands to `sdlc milestone` |
| `sdlc-server` | Add `PATCH /api/milestones/:slug/park` and `/unpark` |
| `MilestonesPage.tsx` | Three sections: Active, Parked (collapsed), Archive |
| `HorizonZone.tsx` | Filter out `status === 'parked'` |
| Tests | ~4 unit tests mirroring existing skip/release patterns |

### What we're NOT building
- Park reason field
- Park/unpark history
- Feature-level parking
- Bulk park operations
- Any change to feature classifier behavior

⚑ Decided: Scope is locked. Minimal, additive, no migration needed.

? Open: After shipping, Jordan should triage the 12 verifying milestones and park the ones that aren't being actively UAT'd.

---

---
session: 1
timestamp: 2026-03-04T03:35:00Z
orientation:
  current: "Problem is clear, design is converged. Simple primitive: parked_at timestamp on Milestone."
  next: "Commit to milestone + feature. Implementation is ~2 hours: Rust struct change, CLI commands, UI sections, REST endpoints."
  commit: "Owner confirms the park/unpark semantics and UI treatment are what they want."
---

**jordan · Owner**
park milestones

Need a way to park milestones

---

**Facilitator**

Let me pull in the team. The brief is one sentence but the problem is real — I count 17 non-released milestones right now, and at most 2-3 are being actively worked. Everything else is clutter. Let's figure out the right primitive.

---

**Felix Wagner · Data Model & CLI Semantics**

I see the shape of this immediately. You already have `Skipped` (permanent cancellation via `skipped_at`) and `Released` (permanent completion via `released_at`). Both are terminal states with timestamps. What's missing is a **reversible pause state**.

The pattern is: `parked_at: Option<DateTime<Utc>>`. When set, `compute_status()` returns `Parked`. When cleared, status reverts to whatever it would have been (Active or Verifying). Same serde treatment as `skipped_at` — `#[serde(default, skip_serializing_if = "Option::is_none")]`.

Priority order in `compute_status()`:
```
Skipped > Released > Parked > Verifying > Active
```

CLI:
```
sdlc milestone park <slug>
sdlc milestone unpark <slug>
```

No flags, no reason field. The reason is always "not now." If someone needs to explain *why*, they can add a comment to the milestone or just... not. The parking action is its own explanation.

**? Open:** Should we also be able to park individual features, or just milestones? Features have `archived: bool` but that's different — archived means "removed from scope", not "paused."

---

**Dana Cho · Product Skeptic**

Let me challenge whether this needs to be built at all.

The problem is dashboard noise — too many milestones competing for attention. Could we solve this with just a filter? A "show only milestones I'm working on this week" toggle?

...Actually, no. The noise isn't just UI. When agents run `/sdlc-prepare` or look at the milestone list to decide what to work on next, parked milestones shouldn't be candidates. A filter is a UI band-aid over a state problem. **The state machine needs to know the difference between "active and in-flight" and "active but not right now."**

This is cheap to build. The `parked_at` timestamp approach reuses the exact same pattern as `skipped_at`. No migration needed (it's an additive field with `serde(default)`). The Rust change is maybe 20 lines. The CLI change is two subcommands. The UI change is moving one filter line in `MilestonesPage.tsx`.

⚑ Decided: This is worth building. It's the right primitive at the right cost.

One thing though — the 12 "verifying" milestones are going to need triage. Some of them (citadel-pantheon-integration, telegram-digest-bot) were committed from ponder sessions before anyone was ready to work on them. Once park exists, Jordan should park those. But that's a one-time cleanup, not a feature concern.

---

**Tobias Krenn · Over-engineering Check**

I want to make sure we don't gold-plate this. Felix's proposal is correct and minimal. Here's what I'd push back on if anyone suggests it:

1. **Do NOT add a "park reason" field.** Nobody will read it. Nobody will update it. It's a form field that makes the `park` command less frictionless for zero value.

2. **Do NOT add park/unpark history tracking.** "Was this parked before?" is a question nobody will ever ask. If they do, `git log` answers it.

3. **Do NOT change how features inside a parked milestone behave.** Features don't know they're in a parked milestone. Their classifier rules don't change. An agent running `/sdlc-next <feature-in-parked-milestone>` still works — it just won't happen because nobody will invoke it. The state machine is already correct.

4. **Do NOT add "bulk park" as a first-class operation.** If Jordan wants to park 5 milestones, five `sdlc milestone park <slug>` commands is fine. Shell history exists.

The total scope should be:
- `Parked` variant in `MilestoneStatus`
- `parked_at` field on `Milestone` struct
- `park()` and `unpark()` methods on `Milestone`
- `compute_status()` updated with Parked priority
- Two CLI subcommands
- Two REST endpoints
- UI: three-section milestones page, filter parked from Horizon
- Tests: 3-4 unit tests matching the existing skip/release test patterns

That's it. If the PR has more than ~200 lines of non-test Rust changes, something went wrong.

---

**Felix Wagner**

Tobias is right on all counts. On the open question about feature-level parking: **don't build it.** Features already have `archived: bool`. The lifecycle for features is: work on it or archive it. Parking is a milestone-level concept because milestones represent *commitment* — "we are doing this body of work." Features are atoms of that commitment. You park the commitment, not the atoms.

⚑ Decided: Parking is milestone-only. No feature-level parking.

---

**Facilitator**

Clean convergence. Let me summarize:

### What we're building

**A `Parked` status for milestones** — a reversible "not now" state that removes milestones from active dashboard views, Horizon zone, and agent consideration, without cancelling them.

### Implementation

| Layer | Change |
|-------|--------|
| `milestone.rs` | Add `Parked` to `MilestoneStatus`, add `parked_at: Option<DateTime<Utc>>`, add `park()` / `unpark()` methods, update `compute_status()` |
| `sdlc-cli` | Add `park` and `unpark` subcommands to `sdlc milestone` |
| `sdlc-server` | Add `PATCH /api/milestones/:slug/park` and `/unpark` |
| `MilestonesPage.tsx` | Three sections: Active, Parked (collapsed), Archive |
| `HorizonZone.tsx` | Filter out `status === 'parked'` |
| Tests | ~4 unit tests mirroring existing skip/release patterns |

### What we're NOT building
- Park reason field
- Park/unpark history
- Feature-level parking
- Bulk park operations
- Any change to feature classifier behavior

⚑ Decided: Scope is locked. Minimal, additive, no migration needed.

? Open: After shipping, Jordan should triage the 12 verifying milestones and park the ones that aren't being actively UAT'd.
