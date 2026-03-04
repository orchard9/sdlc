# Spec: ToolsPage mobile back navigation fix

## Problem

On mobile viewports, `ToolsPage` uses a two-pane layout where the tool list (left) and tool
detail (right) are mutually exclusive. When the user taps a tool, the list pane is hidden
(`hidden md:flex`) and the detail pane is shown. However, there is no back button in the
detail pane to return to the list. Once a tool is selected, mobile users are trapped in the
detail view with no escape path other than navigating away from the page entirely.

## Root cause

`ToolRunPanel` is the detail pane component. It has no `onBack` prop and its header renders
no back affordance. Other pages that share this pattern (EvolvePage, InvestigationPage,
KnowledgePage, GuidelinePage) all pass an `onBack` callback to their detail panels and render
an `ArrowLeft` button that is hidden on `md:` and above.

## Goal

Add a back button to `ToolRunPanel` that is visible only on mobile (`md:hidden`), positioned
in the header row next to the tool name. Pressing it clears the selected tool, which causes
the list pane to become visible again.

## Requirements

1. **`ToolRunPanelProps`** must gain an `onBack: () => void` prop.
2. **Header row** in `ToolRunPanel` must be restructured to a flex row containing:
   - A mobile-only back button (ArrowLeft icon, `md:hidden`)
   - Tool name + description
   - Version badge
3. The back button must call `onBack` on click.
4. **`ToolsPage`** must pass `onBack={() => setSelectedName(null)}` to `ToolRunPanel`.
5. No layout regression on desktop — the button must be `md:hidden` so it is invisible on
   desktop where both panes are always shown.
6. Style must match the pattern used in `EvolvePage` / `InvestigationPage` (same Tailwind
   classes, same `ArrowLeft` icon from `lucide-react`).

## Out of scope

- URL-based routing for tool selection (a larger refactor, tracked separately)
- Any change to the BottomTabBar
- Any change to the tool list pane
</content>
</invoke>