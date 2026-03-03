# ToolsPage Layout Design — Before & After

## The Problem (Current State)

### Desktop (before)
```
┌─────────────────────────────────────────────────────────────────┐
│ Sidebar   │  LEFT PANEL (w-64, 256px)  │  RIGHT PANEL (flex-1)  │
│           │  ┌──────────────────────┐  │  ┌──────────────────┐  │
│ [🔧 Tools]│  │ 🔧 Tools          3  │  │  Tool Name    v1.0 │  │
│ [📋 ]     │  │ ──────────────────── │  │  Description text  │  │
│ [🔍 ]     │  │ ▶ quality-check  ●  │  │                    │  │
│           │  │   ama                │  │  ─────────────────  │  │
│           │  │   audit-debt         │  │  [Run] [History]    │  │
│           │  │                      │  │                    │  │
│           │  │                      │  │  Scope input       │  │
│           │  │                      │  │  [Run ▶]           │  │
│           │  └──────────────────────┘  └──────────────────┘  │
└─────────────────────────────────────────────────────────────────┘
Notes:
  - Left panel: w-64 (256px) — narrower than PonderPage (w-72, 288px)
  - Sidebar already selects first tool on load (auto-select) — hides list intent
  - No browser back button support — /tools stays in URL regardless of selection
  - No deep link: /tools/quality-check doesn't work
  - ToolCard uses onSelect() callback — can't right-click → Open in new tab
```

### Mobile (before) — BROKEN
```
 ┌─────────────┐     ┌─────────────┐
 │  Tools  3   │     │ Tool Name   │
 │ ─────────── │     │ Description │
 │ quality-ch  │  →  │             │
 │ ama         │  tap│ [Run] [Hist]│
 │ audit-debt  │     │ Scope       │
 │             │     │ [Run ▶]     │
 │             │     │             │
 │             │     │  ← NO BACK  │
 └─────────────┘     └─────────────┘
         LIST                DETAIL
                        (USER TRAPPED!)
```

---

## Proposed State (After URL Routing)

### Desktop (after)
```
┌─────────────────────────────────────────────────────────────────┐
│ Sidebar   │  LEFT PANEL (w-72, 288px)  │  RIGHT PANEL (flex-1)  │
│           │  ┌──────────────────────┐  │  ┌──────────────────┐  │
│ [🔧 Tools]│  │ 🔧 Tools          3  │  │  Tool Name    v1.0 │  │
│ [📋 ]     │  │ ──────────────────── │  │  Description text  │  │
│ [🔍 ]     │  │ ▶ quality-check  ●  │  │                    │  │
│           │  │   ama                │  │  ─────────────────  │  │
│           │  │   audit-debt         │  │  [Run] [History]    │  │
│           │  │                      │  │                    │  │
│           │  │                      │  │  Scope input       │  │
│           │  │                      │  │  [Run ▶]           │  │
│           │  └──────────────────────┘  └──────────────────┘  │
└─────────────────────────────────────────────────────────────────┘
Changes from before:
  + Left panel: w-72 (288px) — matches PonderPage exactly
  + URL: /tools/quality-check (deep-linkable, browser history works)
  + ToolCard renders as <Link to="/tools/name"> — right-click → Open in new tab
  + No auto-select: landing on /tools shows list with nothing selected (see empty state)
```

### Desktop — Empty State (no tool selected)
```
┌─────────────────────────────────────────────────────────────────┐
│           │  LEFT PANEL (w-72)          │  RIGHT PANEL           │
│           │  ┌──────────────────────┐  │  ┌──────────────────┐  │
│           │  │ 🔧 Tools          3  │  │                    │  │
│           │  │ ──────────────────── │  │     🔧              │  │
│           │  │   quality-check      │  │  Select a tool     │  │
│           │  │   ama                │  │  to run it         │  │
│           │  │   audit-debt         │  │                    │  │
│           │  │                      │  │                    │  │
│           │  └──────────────────────┘  └──────────────────┘  │
└─────────────────────────────────────────────────────────────────┘
Note: No item pre-selected. User sees the full list first (same as PonderPage).
```

### Mobile (after) — FIXED
```
 Route: /tools              Route: /tools/quality-check
 ┌─────────────┐            ┌─────────────┐
 │ 🔧 Tools  3 │            │ ← Tools     │  ← md:hidden back button
 │ ─────────── │            │ ─────────── │    (ArrowLeft icon)
 │ quality-ch  │   tap  →   │ quality-ch  │
 │ ama         │            │ v1.0        │
 │ audit-debt  │            │ Description │
 │             │            │             │
 │             │  ← back    │ [Run][Hist] │
 │             │  browser   │ Scope input │
 │             │  OR button │ [Run ▶]     │
 └─────────────┘            └─────────────┘
       LIST                      DETAIL
                          ✓ Back button visible
                          ✓ Browser back works
                          ✓ URL is /tools/quality-check
```

---

## Component Changes Summary

### ToolsPage.tsx — main changes

```tsx
// BEFORE
const [selectedName, setSelectedName] = useState<string | null>(null)
const selectedNameRef = useRef(selectedName)

// AFTER
const { name } = useParams<{ name?: string }>()
const navigate = useNavigate()
const selectedTool = tools.find(t => t.name === name) ?? null
const showMobileDetail = !!name  // identical to PonderPage pattern
```

```tsx
// LEFT PANEL: w-64 → w-72
// BEFORE
'w-64 shrink-0 border-r border-border flex flex-col bg-card',
selectedTool ? 'hidden md:flex' : 'flex',

// AFTER  
'w-72 shrink-0 border-r border-border flex flex-col bg-card',
showMobileDetail ? 'hidden md:flex' : 'flex',
```

```tsx
// ToolCard: callback → link
// BEFORE
<ToolCard onSelect={() => setSelectedName(tool.name)} />

// AFTER (idiomatic, enables right-click → Open in new tab)
<Link to={`/tools/${tool.name}`}>
  <ToolCard selected={tool.name === name} />
</Link>
// OR: pass navigate to ToolCard's onSelect — simpler if ToolCard structure is complex
```

```tsx
// REMOVE: auto-select first tool on load
// DELETE THIS BLOCK:
} else if (data.length > 0 && !selectedNameRef.current) {
  setSelectedName(data[0].name)
}
// and selectedNameRef entirely
```

### ToolRunPanel — back button added

```tsx
// Interface
interface ToolRunPanelProps {
  tool: ToolMeta
  onReload: (name?: string) => void
  onBack?: () => void  // ← NEW
}

// In header (before the tool name/description row):
{onBack && (
  <button
    onClick={onBack}
    className="md:hidden flex items-center gap-1.5 mb-3 text-sm text-muted-foreground hover:text-foreground transition-colors"
  >
    <ArrowLeft className="w-4 h-4" />
    Tools
  </button>
)}
```

```tsx
// Called from ToolsPage:
<ToolRunPanel
  tool={selectedTool}
  onReload={(name) => { setLoading(true); load(name) }}
  onBack={() => navigate('/tools')}  // ← NEW
/>
```

### App.tsx — new route

```tsx
// BEFORE
<Route path="/tools" element={<ToolsPage />} />

// AFTER
<Route path="/tools" element={<ToolsPage />} />
<Route path="/tools/:name" element={<ToolsPage />} />
```

---

## Comparison: ToolsPage vs. PonderPage (after fix)

| Dimension              | PonderPage (target) | ToolsPage (after fix) |
|------------------------|--------------------|-----------------------|
| Selection state        | `useParams().slug` | `useParams().name`    |
| Left panel width       | `w-72` (288px)     | `w-72` (288px) ✓     |
| Mobile back nav        | `navigate('/ponder')` | `navigate('/tools')` ✓ |
| Browser back button    | ✓ works            | ✓ works               |
| Deep links             | ✓ `/ponder/:slug`  | ✓ `/tools/:name`      |
| Auto-select on load    | No                 | No (removed) ✓        |
| Empty state            | Shows list         | Shows list ✓          |
| showMobileDetail       | `!!slug`           | `!!name` ✓            |

---

## What "improve both" delivers

- **ToolsPage**: fully aligned with PonderPage pattern (URL routing, back nav, width, no auto-select)  
- **PonderPage**: no changes needed — it already works correctly  
- **"Improve both" interpretation**: ToolsPage rises to PonderPage's quality, making both pages behave consistently
- **Future**: sidebar collapse (desktop) is still a separate improvement, but now both pages are at parity to receive it simultaneously

---

## Blast radius

- Files changed: `ToolsPage.tsx` (main changes) + `App.tsx` (route addition)
- Lines changed: ~50–70
- No other pages touched
- No new components
- No API changes