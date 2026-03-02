---
session: 1
timestamp: 2026-03-01T00:00:00Z
orientation:
  current: "Multiple concrete install blockers identified from Xist's first session — most are quick fixes"
  next: "Triage by severity: SSH install URL (blocker), error filename (quick fix), make install prominence (docs), Vision/Architecture guidance (product)"
  commit: "Clear install guide that works for multi-SSH-key setups, error messages with filenames, and UI guidance for Vision+Architecture creation"
---

## Session 1: Initial Signal Load

Bootstrapped by /sdlc-convo-mine from Discord conversation (sdlc early-user feedback session).

### Signals extracted

- **SSH multi-key install**: The `cargo install --git https://...` URL doesn't work for engineers with multiple SSH keys. Xist found the SSH URL variant by luck. The README needs to lead with this.
- **Permission denied with no filename**: `sdlc init` emits "error: Permission denied (os error 13)" with no context. Every OS error in init should include the path that failed.
- **make install is undiscoverable**: Jordan wrote it but didn't mention it. It should be the first thing in the install instructions.
- **Vision/Architecture not guided**: Once inside the UI, Xist didn't know what Vision and Architecture were or how to create them. The setup screen exists (`/setup`) but wasn't obvious.
- **DEVELOPER.md gap**: Exists but never linked from README or install flow.
- **Update instructions missing**: "When you make changes, how do I get the changed?" — `sdlc update` exists but isn't documented in a visible place.

### Why this might matter

Every new user hits this before they see any value from the tool. One failed install = tool abandoned. Xist is motivated enough to push through. Most people won't be.

### Open questions

- What's the right primary install path? `make install` after clone, or `cargo install --git ssh://...`?
- Should `sdlc init` detect P4/readonly and emit targeted guidance?
- Should the UI's setup screen be the first screen shown, or is it reachable from the dashboard?
- What's the canonical way to communicate "here's how to update" — in the UI, or in a sticky README section?

### Suggested first exploration

Walk through the install flow as if you are a new developer with no prior context. Document every friction point. Fix the error messages in `sdlc init` first (easiest, highest ROI). Then update the README install section.
