---
session: 3
timestamp: 2026-03-02T18:51:00Z
orientation:
  current: "HTML design prototype created for the FeedbackThread UI. Shows: sidebar with Threads nav item, two-pane layout (thread list + thread detail), core element with version history strip, comment thread with agent/human attribution and incorporated badges, compose box, New Thread modal, and V2 action buttons (Synthesize + Promote to Ponder). Design matches existing dark sdlc aesthetic."
  next: "Jordan reviews the design. Confirm nav placement (standalone Threads vs embedded in Ponder) and title-only quick capture. Then spec uat-feedback-threads as a new feature and commit."
  commit: "Design approved → spec uat-feedback-threads feature → /sdlc-ponder-commit"
---

**jordan · Owner**
create an html design so i can see what the need feedback page will look like

---

**Facilitator**

Built the HTML prototype. Here's what the design contains:

## Design walkthrough

### Navigation
Threads is a **standalone nav item** in the sidebar, at the same level as Ponder — directly below it in the Plan section. This answers one of the two open questions from Session 2: threads are not embedded inside Ponder, they have their own top-level presence.

### Two-pane layout
- **Left pane** — thread list with:
  - Thread title (truncated), status badge (open / synthesized / promoted)
  - Comment count + author
  - "New thread" button at top
- **Right pane** — thread detail

### Thread header
Title + status badge + meta line (author, date, comment count). Two action buttons for V2:
- **Synthesize** — triggers agent synthesis run (purple accent)
- **Promote to Ponder** — graduates thread into Ponder workspace (green accent)

### Core element (living body)
Prominent card at the top of the detail view with:
- "Core element" label + pencil icon — makes it clear this is the living part
- Current version indicator (v1 · original)
- Body text — the evolving summary
- **Version strip** at the bottom showing version history chips (v1 active, next synthesis → v2)

⚑ **Decided:** Core element gets a version strip. Current version highlighted, old versions accessible. This directly addresses Dana's trust concern from Session 2 — you can always see what was there before synthesis ran.

### Comment thread
Comments rendered below the core element with:
- Human avatars (initial) vs agent avatars ("ai") in different colors
- **"Absorbed into v1" badge** on incorporated comments (greyed out + dashed border) — visually clear which comments have been synthesized
- Agent comments shown in blue/purple to distinguish them from human comments

⚑ **Decided:** Incorporated comments get visual treatment (reduced opacity + dashed border + "absorbed" badge) not deleted. The history stays visible.

### Compose box
Fixed to the bottom. Shows "Replying as jordan". Cmd+Enter shortcut visible.

### New Thread modal
- Title field (required)
- Core element / body field (optional — answers the title-only question: yes, body can be blank)
- Cancel / Create thread

⚑ **Decided (design):** Title-only quick capture is allowed. Core element defaults to empty body-v1.md that the author can fill in later.

## Open questions remaining

? **Confirmed in design:** Threads is a standalone nav item (not embedded in Ponder). Does Jordan want it differently?

? **Confirmed in design:** Title-only thread creation is supported (core element field is optional). OK?

? **Confirm:** Should the Synthesize and Promote buttons be in V1 (even as disabled) or hidden until V2?

## Design file location

- /tmp/feedback-threads-design.html — open in browser
- Also captured as feedback-threads-ui.html in the ponder scrapbook

**Commit signal still met.** Ready to spec once Jordan confirms nav placement + quick-capture decisions.
