# Code Review: ToolsPage mobile back navigation fix

## Summary

Four targeted changes to `frontend/src/pages/ToolsPage.tsx`. No other files modified.
Build passes (`npm run build` exits 0, no TypeScript errors).

## Changes reviewed

### 1. `ArrowLeft` added to lucide-react import (line 8)

```ts
import { ..., ArrowLeft } from 'lucide-react'
```

Clean. Consistent with how every other page adds icons from the same library.

### 2. `onBack: () => void` added to `ToolRunPanelProps` (line 735)

```ts
interface ToolRunPanelProps {
  tool: ToolMeta
  onReload: (selectName?: string) => void
  onBack: () => void
}
```

Required prop — correct. All callsites already pass it (only one call site exists).
No default needed because the callback is always available from `ToolsPage`.

### 3. Back button in `ToolRunPanel` header (lines 890-896)

```tsx
<button
  onClick={onBack}
  className="md:hidden shrink-0 -ml-1 mt-0.5 p-1 rounded text-muted-foreground hover:text-foreground transition-colors"
  aria-label="Back to tools list"
>
  <ArrowLeft className="w-4 h-4" />
</button>
```

- `md:hidden`: invisible on desktop; no layout regression.
- `shrink-0 -ml-1 mt-0.5`: optical alignment matches the equivalent button in EvolvePage
  and InvestigationPage exactly.
- `aria-label`: accessible label present.
- Hover state (`hover:text-foreground`) and transition present — matches the rest of the UI.

### 4. `onBack` prop passed from `ToolsPage` (line 1336)

```tsx
<ToolRunPanel
  tool={selectedTool}
  onReload={(name) => { setLoading(true); load(name) }}
  onBack={() => setSelectedName(null)}
/>
```

`setSelectedName(null)` is the correct state reset. When `selectedName` becomes `null`,
the ternary `selectedTool ? 'hidden md:flex' : 'flex'` on the left pane flips, restoring
the list. Confirmed by reading the conditional rendering logic in `ToolsPage`.

## Findings

No findings. The change is:
- Minimal (4 lines changed, all in one file + 7 new lines for the button)
- Consistent with the established pattern across 4 other pages
- Fully covered by the QA plan (TC-1 through TC-4)
- No new complexity introduced
- No edge cases: `onBack` is always defined; `setSelectedName` is always available

## Verdict

APPROVED. Ready to advance to audit.
</content>
