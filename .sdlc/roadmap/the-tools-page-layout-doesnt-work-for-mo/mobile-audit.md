# ToolsPage Mobile Layout Audit

## What's broken

### 1. No back navigation (CRITICAL)
When a user taps a tool on mobile:
- The list panel hides correctly ( when selected)
- The detail panel shows correctly ( when selected)
- **But there is no ArrowLeft button to return to the tool list**

The user is trapped. The only escape is a hard reload or tapping the Sidebar nav item.

### 2. ToolRunPanel header has no mobile affordance
 (line 885) renders the tool name/description header but no back arrow.  
Compare: PonderPage, InvestigationPage, EvolvePage, GuidelinePage, KnowledgePage — all have an ArrowLeft back button in the detail panel header.

## What's already correct
- The show/hide toggling logic is correct:
  - `selectedTool ? 'hidden md:flex' : 'flex'` — list hides on mobile when tool selected
  - `!selectedTool ? 'hidden md:flex items-center justify-center' : 'flex flex-col'` — detail shows on mobile when tool selected
- The ToolRunPanel itself has `flex-wrap` on button groups and `overflow-y-auto` — internal layout should survive narrow screens

## Established pattern (from other pages)
PonderPage / InvestigationPage / EvolvePage / GuidelinePage all use:
```tsx
// In the detail panel header, md:hidden
<button onClick={() => setSelectedName(null)} className="md:hidden flex items-center gap-1.5 px-2 py-1.5 rounded hover:bg-muted/60 ...">
  <ArrowLeft className="w-4 h-4" />
  <span className="text-sm">Tools</span>
</button>
```

## Fix

**Minimal, targeted — two changes:**

### Change 1: Add `onBack` prop to `ToolRunPanel`
```tsx
interface ToolRunPanelProps {
  tool: ToolMeta
  onReload: (name: string) => void
  onBack?: () => void  // ← add this
}
```

### Change 2: Render back button in `ToolRunPanel` header (md:hidden)
In the header div (line ~887):
```tsx
<div className="shrink-0 px-5 pt-5 pb-4 border-b border-border/50">
  {onBack && (
    <button
      onClick={onBack}
      className="md:hidden flex items-center gap-1.5 mb-3 text-sm text-muted-foreground hover:text-foreground transition-colors"
    >
      <ArrowLeft className="w-4 h-4" />
      Tools
    </button>
  )}
  <div className="flex items-start gap-3">
    ...
  </div>
</div>
```

### Change 3: Pass `onBack` from `ToolsPage`
```tsx
<ToolRunPanel
  tool={selectedTool}
  onReload={(name) => { setLoading(true); load(name) }}
  onBack={() => setSelectedName(null)}  // ← add this
/>
```

## Scope
- 1 file: `frontend/src/pages/ToolsPage.tsx`
- ~10 lines changed
- No new components, no new state, no API changes
- ArrowLeft already imported in lucide-react for the modals... wait, need to check