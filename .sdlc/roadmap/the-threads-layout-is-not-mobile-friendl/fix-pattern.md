# Fix Pattern: Consistent Mobile Layout

## Tier 1: List/Detail Pages (URL-routed)

All pages with a left list pane and right detail pane should follow the PonderPage pattern.

### Left pane
```tsx
const showMobileDetail = !!slug  // from useParams

<div className={cn(
  'w-72 shrink-0 border-r border-border flex flex-col bg-card',
  showMobileDetail ? 'hidden md:flex' : 'flex',
)}>
```

### Right pane  
```tsx
<div className={cn(
  'flex-1 min-w-0',
  showMobileDetail ? 'flex flex-col' : 'hidden md:flex md:flex-col',
)}>
```

### Back button (in detail header)
```tsx
<button onClick={onBack} className="md:hidden shrink-0 -ml-1 p-1 ..." aria-label="Back">
  <ArrowLeft className="w-4 h-4" />
</button>
```

## Tier 2: Secondary Panel (Files/Workspace)

Two acceptable approaches — both are fine, but pick one per page type and be consistent:

**Option A: Tab Bar** (PonderPage model — better UX, more content real estate)
```tsx
type MobileTab = 'chat' | 'files'

// Tab bar at bottom of detail pane (md:hidden)
<div className="md:hidden shrink-0 flex border-t border-border bg-card">
  <button onClick={() => setMobileTab('chat')}>Chat</button>
  <button onClick={() => setMobileTab('files')}>Files</button>
</div>
```

**Option B: Bottom Sheet** (Evolve/Guideline/Investigation — already working)
- Keep the existing bottom sheet pattern for workspace pages
- The icon badge for artifact count is already there ✓

⚑  Decided: Don't migrate Evolve/Guideline/Investigation bottom sheets to tab bars. They work and cost-of-change is higher than benefit. Fix Threads (broken), fix Tools (no back button). Consistency target is "functional + no hidden truncation" not "pixel-identical".

## Pages to Fix (Priority Order)

1. **ThreadsPage** — CRITICAL. Left pane has no hide logic. 15 min fix.
2. **ToolsPage** — Add back button, prevent auto-select on mobile. 30 min fix.
3. **EvolvePage/GuidelinePage/InvestigationPage** — Already functional. No urgent fix needed.