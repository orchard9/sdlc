# Design: ToolsPage mobile back navigation fix

## Overview

This is a minimal surgical fix. A single `onBack` prop is added to `ToolRunPanel` and a
mobile-only back button is inserted into the existing header row. No routing changes, no
new components, no new files.

## Wireframe (mobile)

```
┌─────────────────────────────────┐
│ ← [tool-name]         v1.0.0   │  ← header row (back button only on mobile)
│   Tool description              │
├─────────────────────────────────┤
│  Usage: ...                     │
│  ...                            │
└─────────────────────────────────┘
```

The back button (`←`) occupies the leftmost slot, `md:hidden`, identical to the pattern
used in `EvolvePage`'s `EntryDetailPane`.

## Changes

### `frontend/src/pages/ToolsPage.tsx`

**1. `ToolRunPanelProps` interface** — add one field:

```ts
interface ToolRunPanelProps {
  tool: ToolMeta
  onReload: (selectName?: string) => void
  onBack: () => void          // NEW
}
```

**2. `ToolRunPanel` signature** — destructure the new prop:

```ts
function ToolRunPanel({ tool, onReload, onBack }: ToolRunPanelProps) {
```

**3. Header row** — add the back button before the tool name div:

Before:
```tsx
<div className="shrink-0 px-5 pt-5 pb-4 border-b border-border/50">
  <div className="flex items-start gap-3">
    <div className="flex-1 min-w-0">
      <h2 className="text-base font-semibold">{tool.display_name}</h2>
      <p className="text-sm text-muted-foreground mt-0.5">{tool.description}</p>
    </div>
    <span className="shrink-0 text-xs font-mono bg-muted/60 border border-border/50 rounded px-2 py-0.5 text-muted-foreground">
      v{tool.version}
    </span>
  </div>
</div>
```

After:
```tsx
<div className="shrink-0 px-5 pt-5 pb-4 border-b border-border/50">
  <div className="flex items-start gap-3">
    <button
      onClick={onBack}
      className="md:hidden shrink-0 -ml-1 mt-0.5 p-1 rounded text-muted-foreground hover:text-foreground transition-colors"
      aria-label="Back to tools list"
    >
      <ArrowLeft className="w-4 h-4" />
    </button>
    <div className="flex-1 min-w-0">
      <h2 className="text-base font-semibold">{tool.display_name}</h2>
      <p className="text-sm text-muted-foreground mt-0.5">{tool.description}</p>
    </div>
    <span className="shrink-0 text-xs font-mono bg-muted/60 border border-border/50 rounded px-2 py-0.5 text-muted-foreground">
      v{tool.version}
    </span>
  </div>
</div>
```

**4. Icon import** — `ArrowLeft` must be added to the existing `lucide-react` import line.

**5. `ToolsPage` — pass `onBack` to `ToolRunPanel`:**

```tsx
<ToolRunPanel
  tool={selectedTool}
  onReload={(name) => { setLoading(true); load(name) }}
  onBack={() => setSelectedName(null)}
/>
```

## Why not URL routing?

URL routing would be the correct long-term architecture, but it is a larger refactor (adds
`useParams`, changes link targets in `ToolCard`, changes browser history behaviour). That
is tracked as a follow-up. The back-button fix is ship-now correct and leaves the routing
refactor as an independent improvement.

## Desktop behaviour (no regression)

`md:hidden` ensures the button is display-none on all viewports `md` (768px) and above.
Desktop users continue to see both panes simultaneously and never interact with the back
button.
</content>
