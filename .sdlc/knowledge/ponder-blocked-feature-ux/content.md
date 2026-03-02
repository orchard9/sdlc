---
session: 1
timestamp: 2026-03-01T00:00:00Z
orientation:
  current: "Blocked features are a dead end in the UI — no options, no navigation, no resolution path"
  next: "Design the blocked-feature UI: what options to surface, how to navigate to dependency, how to accept user instruction override"
  commit: "Implemented blocked feature panel with: blocker description, dependency link, waive option, custom instruction input, and auto-refresh on dependency resolution"
---

## Session 1: Initial Signal Load

Bootstrapped by /sdlc-convo-mine from Discord conversation (sdlc early-user feedback session).

### Signals extracted

- **Run button is a no-op**: When blocked, clicking Run just repeats the blocked message. User expects it to try to clear the blocker.
- **No navigation to dependency**: "Ideally there is a button 'go to the dependency so you can click that button'" — cross-feature navigation is the key missing affordance.
- **No user override**: Xist needed to give the agent a different resolution strategy. There's no text input for this in the UI — had to use Claude Code.
- **Cross-project dependencies**: Xist created two projects that reference each other. The UI doesn't handle this — no cross-project dependency navigation at all.
- **Auto-resolution not detected**: When the dependency was resolved externally, the UI didn't pick it up. (Overlaps with SSE state reliability, but the UX fix here is distinct: a "check now" button or auto-poll.)

### Why this might matter

A blocked feature with no resolution path is a trap. The user can't move forward in the UI and doesn't know what to do. The escape hatch (Claude Code / manual CLI) is not obvious. This is a particularly bad experience for new users who don't know the CLI commands.

### Open questions

- What options should the blocked feature panel show? At minimum: (1) go to dependency, (2) waive blocker, (3) enter custom instruction
- Can the server detect when a blocker is resolved and push an SSE event to update the UI?
- How do cross-project dependencies work in the data model? Is there a `depends_on` field in feature metadata?
- Should "waive" require a reason, or is it one-click?
- What happens to the feature state after a waiver — does it advance, or does it re-evaluate gates?

### Suggested first exploration

Look at how blockers are represented in the sdlc-core state machine (gate.rs, rules.rs) and how the current UI renders the blocked state. Design the blocked-feature panel as a distinct component with explicit action affordances.

---

---
session: 1
timestamp: 2026-03-01T00:00:00Z
orientation:
  current: "Blocked features are a dead end in the UI — no options, no navigation, no resolution path"
  next: "Design the blocked-feature UI: what options to surface, how to navigate to dependency, how to accept user instruction override"
  commit: "Implemented blocked feature panel with: blocker description, dependency link, waive option, custom instruction input, and auto-refresh on dependency resolution"
---

## Session 1: Initial Signal Load

Bootstrapped by /sdlc-convo-mine from Discord conversation (sdlc early-user feedback session).

### Signals extracted

- **Run button is a no-op**: When blocked, clicking Run just repeats the blocked message. User expects it to try to clear the blocker.
- **No navigation to dependency**: "Ideally there is a button 'go to the dependency so you can click that button'" — cross-feature navigation is the key missing affordance.
- **No user override**: Xist needed to give the agent a different resolution strategy. There's no text input for this in the UI — had to use Claude Code.
- **Cross-project dependencies**: Xist created two projects that reference each other. The UI doesn't handle this — no cross-project dependency navigation at all.
- **Auto-resolution not detected**: When the dependency was resolved externally, the UI didn't pick it up. (Overlaps with SSE state reliability, but the UX fix here is distinct: a "check now" button or auto-poll.)

### Why this might matter

A blocked feature with no resolution path is a trap. The user can't move forward in the UI and doesn't know what to do. The escape hatch (Claude Code / manual CLI) is not obvious. This is a particularly bad experience for new users who don't know the CLI commands.

### Open questions

- What options should the blocked feature panel show? At minimum: (1) go to dependency, (2) waive blocker, (3) enter custom instruction
- Can the server detect when a blocker is resolved and push an SSE event to update the UI?
- How do cross-project dependencies work in the data model? Is there a `depends_on` field in feature metadata?
- Should "waive" require a reason, or is it one-click?
- What happens to the feature state after a waiver — does it advance, or does it re-evaluate gates?

### Suggested first exploration

Look at how blockers are represented in the sdlc-core state machine (gate.rs, rules.rs) and how the current UI renders the blocked state. Design the blocked-feature panel as a distinct component with explicit action affordances.

---

---
session: 1
timestamp: 2026-03-01T00:00:00Z
orientation:
  current: "Blocked features are a dead end in the UI — no options, no navigation, no resolution path"
  next: "Design the blocked-feature UI: what options to surface, how to navigate to dependency, how to accept user instruction override"
  commit: "Implemented blocked feature panel with: blocker description, dependency link, waive option, custom instruction input, and auto-refresh on dependency resolution"
---

## Session 1: Initial Signal Load

Bootstrapped by /sdlc-convo-mine from Discord conversation (sdlc early-user feedback session).

### Signals extracted

- **Run button is a no-op**: When blocked, clicking Run just repeats the blocked message. User expects it to try to clear the blocker.
- **No navigation to dependency**: "Ideally there is a button 'go to the dependency so you can click that button'" — cross-feature navigation is the key missing affordance.
- **No user override**: Xist needed to give the agent a different resolution strategy. There's no text input for this in the UI — had to use Claude Code.
- **Cross-project dependencies**: Xist created two projects that reference each other. The UI doesn't handle this — no cross-project dependency navigation at all.
- **Auto-resolution not detected**: When the dependency was resolved externally, the UI didn't pick it up. (Overlaps with SSE state reliability, but the UX fix here is distinct: a "check now" button or auto-poll.)

### Why this might matter

A blocked feature with no resolution path is a trap. The user can't move forward in the UI and doesn't know what to do. The escape hatch (Claude Code / manual CLI) is not obvious. This is a particularly bad experience for new users who don't know the CLI commands.

### Open questions

- What options should the blocked feature panel show? At minimum: (1) go to dependency, (2) waive blocker, (3) enter custom instruction
- Can the server detect when a blocker is resolved and push an SSE event to update the UI?
- How do cross-project dependencies work in the data model? Is there a `depends_on` field in feature metadata?
- Should "waive" require a reason, or is it one-click?
- What happens to the feature state after a waiver — does it advance, or does it re-evaluate gates?

### Suggested first exploration

Look at how blockers are represented in the sdlc-core state machine (gate.rs, rules.rs) and how the current UI renders the blocked state. Design the blocked-feature panel as a distinct component with explicit action affordances.

---

---
session: 1
timestamp: 2026-03-01T00:00:00Z
orientation:
  current: "Blocked features are a dead end in the UI — no options, no navigation, no resolution path"
  next: "Design the blocked-feature UI: what options to surface, how to navigate to dependency, how to accept user instruction override"
  commit: "Implemented blocked feature panel with: blocker description, dependency link, waive option, custom instruction input, and auto-refresh on dependency resolution"
---

## Session 1: Initial Signal Load

Bootstrapped by /sdlc-convo-mine from Discord conversation (sdlc early-user feedback session).

### Signals extracted

- **Run button is a no-op**: When blocked, clicking Run just repeats the blocked message. User expects it to try to clear the blocker.
- **No navigation to dependency**: "Ideally there is a button 'go to the dependency so you can click that button'" — cross-feature navigation is the key missing affordance.
- **No user override**: Xist needed to give the agent a different resolution strategy. There's no text input for this in the UI — had to use Claude Code.
- **Cross-project dependencies**: Xist created two projects that reference each other. The UI doesn't handle this — no cross-project dependency navigation at all.
- **Auto-resolution not detected**: When the dependency was resolved externally, the UI didn't pick it up. (Overlaps with SSE state reliability, but the UX fix here is distinct: a "check now" button or auto-poll.)

### Why this might matter

A blocked feature with no resolution path is a trap. The user can't move forward in the UI and doesn't know what to do. The escape hatch (Claude Code / manual CLI) is not obvious. This is a particularly bad experience for new users who don't know the CLI commands.

### Open questions

- What options should the blocked feature panel show? At minimum: (1) go to dependency, (2) waive blocker, (3) enter custom instruction
- Can the server detect when a blocker is resolved and push an SSE event to update the UI?
- How do cross-project dependencies work in the data model? Is there a `depends_on` field in feature metadata?
- Should "waive" require a reason, or is it one-click?
- What happens to the feature state after a waiver — does it advance, or does it re-evaluate gates?

### Suggested first exploration

Look at how blockers are represented in the sdlc-core state machine (gate.rs, rules.rs) and how the current UI renders the blocked state. Design the blocked-feature panel as a distinct component with explicit action affordances.

---

---
session: 1
timestamp: 2026-03-01T00:00:00Z
orientation:
  current: "Blocked features are a dead end in the UI — no options, no navigation, no resolution path"
  next: "Design the blocked-feature UI: what options to surface, how to navigate to dependency, how to accept user instruction override"
  commit: "Implemented blocked feature panel with: blocker description, dependency link, waive option, custom instruction input, and auto-refresh on dependency resolution"
---

## Session 1: Initial Signal Load

Bootstrapped by /sdlc-convo-mine from Discord conversation (sdlc early-user feedback session).

### Signals extracted

- **Run button is a no-op**: When blocked, clicking Run just repeats the blocked message. User expects it to try to clear the blocker.
- **No navigation to dependency**: "Ideally there is a button 'go to the dependency so you can click that button'" — cross-feature navigation is the key missing affordance.
- **No user override**: Xist needed to give the agent a different resolution strategy. There's no text input for this in the UI — had to use Claude Code.
- **Cross-project dependencies**: Xist created two projects that reference each other. The UI doesn't handle this — no cross-project dependency navigation at all.
- **Auto-resolution not detected**: When the dependency was resolved externally, the UI didn't pick it up. (Overlaps with SSE state reliability, but the UX fix here is distinct: a "check now" button or auto-poll.)

### Why this might matter

A blocked feature with no resolution path is a trap. The user can't move forward in the UI and doesn't know what to do. The escape hatch (Claude Code / manual CLI) is not obvious. This is a particularly bad experience for new users who don't know the CLI commands.

### Open questions

- What options should the blocked feature panel show? At minimum: (1) go to dependency, (2) waive blocker, (3) enter custom instruction
- Can the server detect when a blocker is resolved and push an SSE event to update the UI?
- How do cross-project dependencies work in the data model? Is there a `depends_on` field in feature metadata?
- Should "waive" require a reason, or is it one-click?
- What happens to the feature state after a waiver — does it advance, or does it re-evaluate gates?

### Suggested first exploration

Look at how blockers are represented in the sdlc-core state machine (gate.rs, rules.rs) and how the current UI renders the blocked state. Design the blocked-feature panel as a distinct component with explicit action affordances.
