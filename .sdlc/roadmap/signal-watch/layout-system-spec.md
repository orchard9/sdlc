# Layout System Spec — Desktop & Mobile

> Canonical layout grid for the SDLC app. All pages must conform to this spec.
> This is the foundation all future layout work builds on.

## Desktop Layout (≥768px)

The app shell is a horizontal flex layout with **five named zones**:

```
┌──────────┬────────────┬─────────────────┬───────────┬──────────────────┐
│ NAV      │ CONTEXT    │ CONTENT          │ ARTIFACTS │ AGENT ACTIVITY   │
│ (LEFT)   │ (page-     │ (flex-1)         │ (right1)  │ (right2)         │
│          │  specific) │                  │           │                  │
└──────────┴────────────┴─────────────────┴───────────┴──────────────────┘
```

### Zone definitions

| Zone | Current width | Target default | Min | Max | Collapsible | Resizable |
|---|---|---|---|---|---|---|
| **NAV** (Sidebar) | `w-56` (224px) | 224px | 180px | 280px | ✅ YES | — |
| **CONTEXT** (page list) | `w-72` (288px) | 240px | 180px | 360px | ✅ YES | ✅ YES |
| **CONTENT** | `flex-1` | flex-1 | 320px | — | — | — |
| **ARTIFACTS** (workspace) | `w-64` (256px) | 256px | 200px | 400px | ✅ YES | ✅ YES |
| **AGENT ACTIVITY** | `w-72` (288px) | 288px | 240px | 480px | ✅ YES | ✅ YES |

### Which zones appear on which pages

- **All pages**: NAV + CONTENT + AGENT ACTIVITY
- **Ponder/Investigate/Evolve/Guidelines detail**: NAV + CONTEXT + CONTENT + ARTIFACTS + AGENT ACTIVITY
- **Feature detail**: NAV + CONTENT + AGENT ACTIVITY (no CONTEXT or ARTIFACTS)

### Budget audit (current vs target, Ponder page at 1440px)

| State | Fixed chrome | Content width |
|---|---|---|
| **Current** (all open, fixed) | 1056px | 384px — BROKEN |
| **Target default** (NAV + CONTEXT + ARTIFACTS collapsed, AGENT open) | 288px | 1152px |
| **Target full** (all open, target widths) | 1008px | 432px — still tight |
| **Power user** (only NAV collapsed) | 784px | 656px |

⚑ Decided: **Default state should open with NAV + AGENT ACTIVITY visible, CONTEXT collapsed**. Users reveal CONTEXT when they select an entry.

### Collapse behavior

- **NAV collapse**: full hide → icon-only rail (40px) → full hide. Toggle button at bottom of sidebar.
- **CONTEXT collapse**: full hide. The page title row (or a left-edge handle) toggles it.
- **ARTIFACTS collapse**: full hide. Toggle button at right edge of CONTENT area.
- **AGENT ACTIVITY collapse**: full hide → narrow open button strip (32px). Already implemented.

### Resize behavior

- CONTEXT: drag handle on right edge
- ARTIFACTS: drag handle on left edge
- AGENT ACTIVITY: drag handle on left edge
- Implementation: `react-resizable-panels` (Brian Vaughn) — the standard for this in React. Used in VS Code web, StackBlitz, CodeSandbox. Avoids 200+ lines of manual pointer-event code.
- Persistence: `localStorage` under keys `sdlc.panel.nav`, `sdlc.panel.context`, `sdlc.panel.artifacts`, `sdlc.panel.agent`

---

## Mobile Layout (<768px)

Mobile gets a **completely different mental model**: bottom tabs with sheets/drawers. No horizontal columns.

```
┌─────────────────────────────┐
│  CONTENT (full width)        │
│                              │
│                              │
│                              │
│                              │
├─────────────────────────────┤
│  [Work] [Plan] [Agent] [Nav] │  ← Bottom tab bar
└─────────────────────────────┘
```

### Mobile zones

| Zone | Behavior |
|---|---|
| **Content** | Full screen, current page |
| **NAV** | Bottom sheet drawer (slides up from bottom) |
| **CONTEXT** | Full-screen sheet (back button returns to content) |
| **ARTIFACTS** | Full-screen sheet (tab inside CONTEXT detail) |
| **AGENT ACTIVITY** | FAB → bottom sheet drawer (already has `AgentPanelFab`) |

### Mobile navigation model

For Ponder (and Investigate/Evolve/Guidelines), the mobile flow is:
1. `/ponder` → Shows entry list (full screen)
2. Tap entry → CONTENT slides in (full screen), back button returns to list
3. Files tab button → ARTIFACTS slides in as full-screen sheet
4. Agent FAB → AGENT ACTIVITY slides up as bottom drawer

⚑ Decided: **Mobile does not share the horizontal panel system.** The `hidden md:flex` / `md:hidden` pattern is correct — mobile and desktop are parallel implementations.

### Bottom tab bar (persistent on mobile)

Current `AppShell` has no bottom tab bar on mobile. We need one for Ponder navigation:
```
[Work: Dashboard/Milestones/Features] [Plan: Ponder/Investigate/Evolve] [Activity: Agent] [More: Nav]
```
This maps to the sidebar's group labels (work, plan, setup).

---

## Implementation order

1. **AppShell foundation**: NAV collapse, AGENT collapse+resize (already partially done), localStorage persistence
2. **Ponder-specific**: CONTEXT resize+collapse, ARTIFACTS resize+collapse
3. **Mobile**: Bottom tab bar, CONTEXT/ARTIFACTS as full-screen sheets
4. **Global**: All other pages inherit AppShell improvements automatically