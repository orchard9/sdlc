# Review: Vision and Architecture Guidance in Setup

## Summary

Three additive changes across two frontend files and the README. No backend changes. TypeScript compiles cleanly (`tsc --noEmit` passes with zero errors).

## Changes Reviewed

### README.md — "First steps" section

Inserted after the "Initialize a project" section, before "Create a feature". Content matches the spec exactly:
- `sdlc ui` code block
- Bullet for Vision with agent-use explanation
- Bullet for Architecture with agent-use explanation
- Closing sentence directing users to create features

No formatting issues. Fits naturally in the Quickstart flow.

### frontend/src/pages/SetupPage.tsx — Vision subtitle (step 2)

Old text: "Edit the generated vision or write your own. VISION.md tells agents what you're building and why."

New text: "Explain why this project exists and who it serves. VISION.md is read by every AI agent to make decisions that stay aligned with your goals. Edit the generated draft or write your own."

Improvement: clarifies the purpose (why it exists, who it serves) and the agent-alignment value before giving the editing instruction. Style matches surrounding text.

### frontend/src/pages/SetupPage.tsx — Architecture subtitle (step 3)

Old text: "Edit the generated architecture or write your own. ARCHITECTURE.md maps your tech stack and key components."

New text: "Describe how the system works — key components, tech stack, and constraints. ARCHITECTURE.md tells agents what's in scope. Edit the generated draft or write your own."

Improvement: leads with what to describe rather than what to do with the file, and the agent-scope value is explicit. Style matches.

### frontend/src/pages/Dashboard.tsx — Vision/Architecture missing banner

- State variable renamed from `setupIncomplete` to `missingVisionOrArch` — more precise name.
- `Promise.all` now fetches config + vision + arch (removed `getProjectAgents` call — team absence no longer drives the banner).
- Condition changed from `(!vision?.exists && !arch?.exists)` to `!vision?.exists || !arch?.exists` — now fires if *either* is missing, not only when both are missing. This is the correct behavior per the spec.
- Banner text updated from generic "Project setup is incomplete" to specific "Vision or Architecture not defined — agents use these to make aligned decisions."

## Findings

### ACCEPTED — Condition is `||` not `&&`

The original condition (`!vision?.exists && !arch?.exists`) only fired when *both* were missing. The new condition (`!vision?.exists || !arch?.exists`) fires when *either* is missing. This is intentionally more sensitive and matches the spec: if only Architecture is missing, agents still lack a key document. Correct behavior.

### FYI — Additional pre-existing Dashboard changes in diff

The diff shows unrelated pre-existing changes also in Dashboard.tsx: `DashboardEmptyState` component import, `Key` icon removal, `EscalationIcon` for `secret_request` changed to `AlertTriangle`. These are pre-existing from other work and not introduced by this feature — no action needed.

## Result

All three spec tasks implemented correctly. TypeScript clean. No regressions.
