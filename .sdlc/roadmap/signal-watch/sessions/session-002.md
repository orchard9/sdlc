---
session: 2
timestamp: 2026-03-01T21:00:00Z
orientation:
  current: "Layout design pivot — full spec for desktop/mobile layout system and Ponder-specific UI with collapsible/resizable panels"
  next: "sdlc-ponder-commit signal-watch — promote to milestone v15-layout-foundation with two features: layout-appshell-panels and layout-ponder-columns"
  commit: "All five zones specced with widths, collapse/resize behaviors, mobile strategy, and acceptance criteria — READY to commit"
---

## Session 2: Layout Design Pivot

**jordan · Owner**
we want to change this entire task into layout design. We need to consciously organize desktop layouts and mobile layouts so that we can set a foundation to build really effectively layouts. after that we want to focus on the "ponder" ui for both desktop and mobile to make it really good.

The biggest upgrade we can do in ponder is handle the multiple columns while also fitting seamlessly into the site and not disrupting anything.

The comment about collapsing/resizing is important, so the left menu, then the ponder left menu, then the content, then the artifact bar, then the agent activity bar.

We at minimum have to have our permanent left and right bars collapsable (left menu and agent activity bar)
and our agent activity bar we should be able to resize
then the ponder bar and the artifact bar we should be able to resize

---

**Facilitator note:** This session pivots signal-watch away from the original "weak signals parking lot" purpose and focuses it entirely on layout design. The original brief's "Column resizing" signal was the seed — it has now grown into a full layout system initiative. The other weak signals (arrow keys bug, links not working, etc.) will be left behind or promoted separately.

---

### The numbers problem (Facilitator reads the code)

Before bringing in thought partners, let's anchor on the actual constraint.

Current Ponder detail page column widths (from code):
- Sidebar (`w-56`): **224px**
- Ponder list (`w-72`): **288px**
- WorkspacePanel (`w-64`): **256px**
- AgentPanel (`w-72`): **288px**
- **Total fixed chrome: 1056px**
- **Content at 1440px: 384px** — roughly the width of an iPhone Plus in landscape

That's the real problem. Not missing features. A layout budget that was never managed consciously.

---

**Ben Hartley · Developer Productivity UX**

The 384px content width confirms it. That's why the chat input feels broken — you're typing into a phone-width container inside a desktop app. IDE panel systems solve this by defaulting to a single "work area" and letting users summon supporting panels as needed. The wrong default is "everything open." The right default is "content visible, panels available."

I want to push on the collapse behavior. There's a difference between "hidden" and "collapsed to a rail." The sidebar is navigation — if you collapse it to nothing, users lose their location context. A 40px icon rail keeps them oriented without consuming real estate. The agent panel can go to nothing because it has a visible toggle button.

For keyboard shortcuts — this is non-optional for developer tools. VS Code uses `⌘ B` for sidebar, `⌘ J` for terminal. We can use `⌘ \` (sidebar) and `⌘ ⇧ A` (agent) to not conflict. Power users will use these within the first hour. The visual toggles are for discovery, not for repeated use.

**?  Open: Should CONTEXT (ponder list) default to collapsed or open when arriving at a detail page?**

I'd argue collapsed — you came from the list, you already selected what you want. The list is a "go back and pick something else" affordance, not a persistent navigation panel. Make it available, not default-visible.

---

**Nadia Osei · Rich Document Experience**

Agreed on the budget problem. But I want to focus on what collapsing buys you in the Ponder context specifically.

The Ponder page has three very different reading modes:
1. **Chat mode**: You're in dialogue with the agent. You need the full content area and nothing else.
2. **Artifact review mode**: You've generated something and you want to read it. You need ARTIFACTS open and CONTEXT collapsed.
3. **Multi-ponder mode**: You're jumping between ponders rapidly. You need CONTEXT open and ARTIFACTS collapsed.

None of these modes is served by the current "all open, all cramped" default. The right answer is **mode-awareness** — the UI should bias toward making it easy to enter each mode.

For the WorkspacePanel (ARTIFACTS), I'd add: when an agent produces a new artifact, the panel should **auto-expand** if it was collapsed. The content pushed it open. If you didn't want it, you close it. This is the "notification" pattern — open on new content, user closes when done.

⚑ Decided: **ARTIFACTS auto-expands when a new scrapbook artifact is produced.** Requires an SSE event hook.

---

**Tobias Krenn · Skeptical Engineering Lead**

I want to name the scope risk before we're 3 weeks into this and someone says "let's add snap-to-grid layouts."

The minimal spec is:
1. Sidebar: collapse/expand. Done. ~80 lines of React + CSS.
2. AgentPanel: resize + collapse. ~40 lines wrapping existing component.
3. PonderPage CONTEXT: resize + collapse. ~40 lines.
4. PonderPage ARTIFACTS (WorkspacePanel): resize + collapse. ~40 lines.

That's the complete feature. Everything else — floating panels, detach to window, saved workspace layouts, per-page layout memory — is scope creep. Draw the line here and defend it.

On `react-resizable-panels`: I looked at the alternatives. Manual pointer-event implementation is ~200 lines and you'll spend 2 days on edge cases (pointer capture, mobile touch, keyboard accessibility). The library is 4KB gzipped and battle-tested in VS Code Web. Use the library.

One concern: `react-resizable-panels` uses percentage-based sizing internally. If we want to persist absolute pixel widths, we need a conversion layer. Not hard — `panelWidth = percentage * containerWidth` — but needs to be in the spec.

⚑ Decided: **Use `react-resizable-panels`.** Percentage internally, converted to px for localStorage via `onLayout` callback.

**?  Open: Do we persist panel sizes globally or per-ponder-entry?**

Global is simpler and almost certainly correct. The user's preferred layout is a personal preference, not document state.

---

**Xist · User (Enterprise Game Dev)**

Speaking from the original experience: the right columns being "unusable" was specifically the agent activity panel and whatever was next to it. When the agent is running and I want to watch it, I maximize that panel. When I'm writing chat input I want the content area. There's no mode where I want all of them small at once.

What would have helped: if the agent panel could be **maximized** easily — like take up 50% of the screen — not just 288px fixed. That's the resize case. Same for the artifact panel: when I'm reviewing a generated document, I want it to take up most of the right side.

Also on mobile: I was testing on my phone at some point. The columns completely broke. You'd scroll right and see half a panel. A proper mobile layout with tabs is the right call — I'd actually use it on the go.

---

### Synthesis and decisions

**⚑ Decided: Layout is five named zones** (NAV, CONTEXT, CONTENT, ARTIFACTS, AGENT). Documented in `layout-system-spec.md`.

**⚑ Decided: Default state for Ponder detail** — CONTEXT collapsed, ARTIFACTS collapsed, AGENT open. User reveals panels as needed.

**⚑ Decided: NAV collapses to 40px icon rail** (not full hide — preserves location context per Ben's concern).

**⚑ Decided: AGENT ACTIVITY collapses to 32px strip** (already close to this; existing collapse button).

**⚑ Decided: Use `react-resizable-panels`** for drag-resize implementation.

**⚑ Decided: Panel sizes persist to localStorage** globally (not per-entry). Percentage internally, px at rest.

**⚑ Decided: ARTIFACTS auto-expands when new artifact produced** (SSE-triggered via useSSE hook).

**⚑ Decided: Mobile gets a different model** — bottom tabs, full-screen sheets, no horizontal columns. Ponder detail gets Chat/Files/Team inner tab bar.

**?  Open: Keyboard shortcut conflicts** — need to verify `⌘\`, `⌘]`, `⌘[`, `⌘⇧A` aren't already bound in AppShell.tsx.

**?  Open: Icon-rail for NAV** — does collapsed rail need tooltips on hover? Probably yes — polish detail, not blocker.

**?  Open: Should Investigate/Evolve/Guidelines pages** get the same inner tab bar treatment as Ponder? Likely yes — but scope to Ponder first.

---

### Mobile layout decisions

**⚑ Decided: No app-level bottom tab bar** for now. Current mobile header with back-button is fine for most pages.

**⚑ Decided: Ponder detail mobile gets an inner tab bar** (Chat | Files | Team) replacing the current `md:hidden` WorkspacePanel hack.

---

### What this is NOT (scope guard)

- ❌ Not a floating panel system
- ❌ Not per-page layout memory (a single global preference)
- ❌ Not animated physics-based panels
- ❌ Not a responsive grid system replacing Tailwind
- ❌ Not mobile-first refactor of the entire app — just Ponder detail

---

### Feature scope for commitment

This work naturally splits into two features:
1. **`layout-appshell-panels`** — AppShell level: Sidebar collapse-to-rail, AgentPanel resize+collapse. Foundation for all pages.
2. **`layout-ponder-columns`** — Ponder-specific: CONTEXT resize+collapse, ARTIFACTS resize+collapse, mobile inner tab bar.

Sequential — AppShell first, Ponder builds on it. Both belong in a new milestone: **`v15-layout-foundation`**.

See scrapbook artifacts:
- `layout-system-spec.md` — complete zone table, widths, behaviors
- `ponder-layout-spec.md` — Ponder-specific component changes and acceptance criteria
