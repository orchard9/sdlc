---
session: 1
timestamp: 2026-03-04T00:00:00Z
orientation:
  current: "Raw signal from conversation — copy/screenshot UX buttons requested but not yet shaped into a problem statement"
  next: "Interrogate the brief: what surfaces need buttons, what's technically feasible for screenshot capture, and who benefits most?"
  commit: "Clear problem statement + user stories for copy and screenshot affordances + rough implementation direction"
---

## Session 1: Initial Signal Load

Bootstrapped by /sdlc-convo-mine from a team conversation dump.

### Signals extracted

- User explicitly requested "click to copy" for prompts and artifacts — both raw markdown and rendered form
- Enthusiastic request for per-artifact screenshot button ("💰") — currently users resort to OS screenshot keybindings so often they have them memorized
- Framed as both personal productivity AND lowering the barrier for other users to contribute feedback
- The motivation is twofold: (1) personal workflow efficiency, (2) enabling less technical teammates to easily share what they're seeing

### Why this might matter

The sdlc UI surfaces a lot of valuable artifacts — specs, designs, ponder sessions, command outputs. Right now, sharing any of these externally (Slack, email, feedback) requires manually selecting text or taking OS screenshots. One-click copy and one-click screenshot would make the UI a much better collaboration surface. The screenshot button specifically is novel — capturing a rendered view preserves formatting and visual context that raw markdown loses.

### Open questions

- Which surfaces are highest priority? (Ponder dialogue messages, artifact files, command code blocks, spec/design pages)
- Is per-component screenshot feasible? html2canvas and dom-to-image libraries exist but have edge cases with custom fonts, shadows, and dynamic content
- Should copy produce raw markdown, rendered HTML, or both (with toggle)?
- Should the screenshot button download a file, copy to clipboard, or open in a new tab?
- Is there a standard UX pattern (hover reveal vs. always visible icon) that fits the existing design?

### Suggested first exploration

Start by auditing which artifact surfaces users interact with most — ponder dialogue messages and artifact file views are likely the top two. Prototype the copy button first (technically simpler, immediately useful) then evaluate screenshot feasibility. Ask: is dom-to-image/html2canvas acceptable given our dark theme and custom components?
