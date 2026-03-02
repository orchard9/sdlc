# Ponder UI — Desktop & Mobile Spec

> Specific behavior spec for the Ponder page layout. Depends on the Layout System Spec.

## The problem in numbers

On a 13" MacBook Pro (1440×900):
- Current fixed chrome: Sidebar (224) + Ponder list (288) + Workspace (256) + AgentPanel (288) = **1056px**
- Content area left over: **384px** — barely wider than a phone screen
- Result: The chat input is unusable. The artifact viewer is compressed. The agent activity cards are unreadable.

## Ponder column spec (desktop)

### Default open state

When navigating to `/ponder/:slug`:
```
[NAV 224px] | [CONTEXT collapsed] | [CONTENT flex-1] | [ARTIFACTS collapsed] | [AGENT open 288px]
```
- User sees the full content width plus agent activity
- CONTEXT (entry list) revealed via left-edge toggle button or list icon in header
- ARTIFACTS revealed via "Files" icon button in the content header (already exists)

### Expanded state (all open)

```
[NAV collapsed to rail 40px] | [CONTEXT 240px] | [CONTENT flex-1] | [ARTIFACTS 256px] | [AGENT 288px]
```

### Resize drag handles

- CONTEXT right edge: grab → drag → release. Snap points at 180, 240, 320px.
- ARTIFACTS left edge: grab → drag → release. Snap points at 200, 256, 360px.
- AGENT left edge: already partially exists; add min 240px / max 480px.

### Keyboard shortcuts

- `⌘ \` — toggle NAV sidebar
- `⌘ ]` — toggle CONTEXT panel (ponder list)
- `⌘ [` — toggle ARTIFACTS panel
- `⌘ ⇧ A` — toggle AGENT panel

These follow VS Code / JetBrains conventions that power users already have in muscle memory.

---

## Ponder layout on mobile

### Entry list view (`/ponder`)
- Full-screen scrollable list of ponder entries
- "New ponder" FAB in bottom-right
- No column layout — just a vertical list

### Detail view (`/ponder/:slug`)
- Full screen with the DialoguePanel (chat)
- **Tab bar** at the top of the content area (not the app tab bar):
  - `Chat` | `Files` | `Team`
  - Swipe or tap to switch
- "Chat" is default — what users want 90% of the time
- "Files" shows WorkspacePanel full-screen (artifact viewer)
- "Team" shows team members, advisory controls

### Agent activity on mobile
- The existing `AgentPanelFab` + bottom sheet is correct
- No changes needed here

---

## Component changes required

### AppShell.tsx
- Add collapse toggle to Sidebar (left edge button, persists to localStorage)
- Sidebar collapse → icon rail (40px wide, just icons, no labels)
- Replace fixed `w-72` on AgentPanel with resizable panel

### AgentPanel.tsx
- Add resize drag handle on left edge
- Persist width to localStorage(`sdlc.panel.agent`)
- min-width 240px, max-width 480px

### Sidebar.tsx
- Add collapsed state (icon-only rail at 40px)
- Collapse toggle button at bottom
- Persist to localStorage(`sdlc.panel.nav`)

### PonderPage.tsx (detail view)
- Wrap CONTEXT (entry list, currently `w-72`) in resizable panel
- Wrap ARTIFACTS (WorkspacePanel, currently `w-64`) in resizable panel
- Add collapse toggles for each
- Mobile: add inner tab bar (Chat/Files/Team)

### react-resizable-panels integration
- `npm install react-resizable-panels`
- Used as: `<PanelGroup direction="horizontal"><Panel> ... <PanelResizeHandle> ... <Panel>`
- Built-in keyboard resize, ARIA, and pointer handling
- Size stored as percentage (0-100), convert to px for localStorage

---

## What NOT to build (scope guard)

- ❌ Don't build a floating/detachable panel system
- ❌ Don't make CONTENT area resizable — it's always flex-1
- ❌ Don't add resize to pages that don't have multiple columns (Dashboard, Features list, Milestones)
- ❌ Don't implement panel snap grids or saved layouts — too much complexity for now
- ❌ Don't animate panel open/close with physics — CSS transition 200ms is sufficient

---

## Acceptance criteria

1. On desktop, Sidebar can be collapsed to icon rail and re-expanded. Width persists across page loads.
2. On desktop, AgentPanel can be resized by dragging its left edge. Width persists.
3. On Ponder detail: the entry list (CONTEXT) can be collapsed and resized.
4. On Ponder detail: the WorkspacePanel (ARTIFACTS) can be collapsed and resized.
5. On mobile, Ponder detail shows Chat/Files/Team tab bar. No horizontal columns visible.
6. No horizontal scrollbar appears at any reasonable viewport width.
7. Content area never narrows below 320px before columns start collapsing.