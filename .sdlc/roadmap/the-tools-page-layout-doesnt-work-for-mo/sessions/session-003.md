---
session: 3
timestamp: 2026-03-03T02:54:00Z
orientation:
  current: "Design produced. URL routing decision solid. Visual spec shows before/after for desktop and mobile — concrete code diffs included. ~50-70 lines, two files."
  next: "If Jordan approves the design, create feature fix-toolspage-url-routing and run /sdlc-next to implement it."
  commit: "Architecture locked in Session 2. Design produced in Session 3. Ready to build when Jordan signs off on the layout."
---

**jordan · Owner**
show me a web design

---

## Session 3 — Visual Design

Context from Sessions 1–2:
- Session 1: diagnosed missing mobile back button, proposed minimal fix.
- Session 2: Jordan rejected minimal fix. Architecture pivoted to URL routing (`/tools/:name`) matching PonderPage's established pattern. Decision locked.

Jordan's new request: **"show me a web design."** They want to see what this looks like before building.

Thought partners: Ben Hartley (UX), Tobias Krenn (scope skeptic), Dan Reeves (minimalist).

---

**Ben Hartley:**
> Show them before/after. Mobile first — that's where it's broken. Then desktop so they can see the width alignment. Include the code delta so they understand the blast radius. Designers who hide implementation cost are lying.

**Tobias Krenn:**
> And call out what we're explicitly NOT changing. PonderPage untouched. No shared component. That's the most important decision and the design should make it visible.

---

### BEFORE — Desktop
```
┌────────────────────────────────────────────────────────────────┐
│ Sidebar  │  LEFT PANEL (w-64 = 256px)  │  RIGHT PANEL          │
│          │ ┌────────────────────────┐  │ ┌──────────────────┐  │
│ [🔧]     │ │ 🔧 Tools            3  │  │ │ quality-check    │  │
│          │ │ ────────────────────── │  │ │ v1.0             │  │
│          │ │ ▶ quality-check   ●   │  │ │ Description text │  │
│          │ │   ama                  │  │ │                  │  │
│          │ │   audit-debt           │  │ │ [Run] [History]  │  │
│          │ │                        │  │ │ Scope input      │  │
│          │ └────────────────────────┘  │ │ [Run ▶]          │  │
│          │                             │ └──────────────────┘  │
└────────────────────────────────────────────────────────────────┘
  URL: /tools (always)

Problems:
  ✗ w-64 — narrower than PonderPage (w-72), visual jolt when switching pages
  ✗ First tool auto-selected on load — user never consciously chooses
  ✗ URL never updates — no browser history, no deep links
  ✗ ToolCard is a button — can't right-click → Open in new tab
```

### BEFORE — Mobile (BROKEN)
```
 Route: /tools (frozen)
 ┌──────────────────┐        ┌──────────────────┐
 │ 🔧 Tools       3 │        │ quality-check    │
 │ ─────────────── │        │ v1.0             │
 │ quality-check   │  tap   │ Description text │
 │ ama             │  ───▶  │                  │
 │ audit-debt      │        │ [Run] [History]  │
 │                 │        │ Scope input      │
 │                 │        │ [Run ▶]          │
 │                 │        │                  │
 │                 │   ???  │  ← NO BACK BTN   │
 └──────────────────┘        └──────────────────┘
  LIST                        DETAIL — USER TRAPPED
```

---

### AFTER — Desktop
```
┌────────────────────────────────────────────────────────────────┐
│ Sidebar  │  LEFT PANEL (w-72 = 288px)  │  RIGHT PANEL          │
│          │ ┌────────────────────────┐  │ ┌──────────────────┐  │
│ [🔧]     │ │ 🔧 Tools            3  │  │ │ quality-check    │  │
│          │ │ ────────────────────── │  │ │ v1.0             │  │
│          │ │ ▶ quality-check   ●   │  │ │ Description text │  │
│          │ │   ama                  │  │ │                  │  │
│          │ │   audit-debt           │  │ │ [Run] [History]  │  │
│          │ │                        │  │ │ Scope input      │  │
│          │ └────────────────────────┘  │ │ [Run ▶]          │  │
│          │                             │ └──────────────────┘  │
└────────────────────────────────────────────────────────────────┘
  URL: /tools/quality-check

  ✓ w-72 — matches PonderPage exactly (32px wider, no visual jolt)
  ✓ ToolCard is a <Link> — right-click → Open in new tab works
  ✓ Browser back/forward work natively (real navigation)
  ✓ Deep-linkable: /tools/quality-check opens directly
```

### AFTER — Desktop (empty state / no tool selected)
```
┌────────────────────────────────────────────────────────────────┐
│ Sidebar  │  LEFT PANEL (w-72)          │  RIGHT PANEL           │
│          │ ┌────────────────────────┐  │                        │
│ [🔧]     │ │ 🔧 Tools            3  │  │                        │
│          │ │ ────────────────────── │  │       🔧               │
│          │ │   quality-check        │  │  Select a tool         │
│          │ │   ama                  │  │  to run it             │
│          │ │   audit-debt           │  │                        │
│          │ └────────────────────────┘  │                        │
└────────────────────────────────────────────────────────────────┘
  URL: /tools

  ✓ No auto-select — user sees full list, makes a conscious choice
  ✓ Same behavior as PonderPage landing on /ponder
```

### AFTER — Mobile (FIXED)
```
 Route: /tools                  Route: /tools/quality-check
 ┌──────────────────┐           ┌──────────────────┐
 │ 🔧 Tools       3 │           │ ← Tools          │ ← md:hidden
 │ ─────────────── │           │ ─────────────── │   ArrowLeft btn
 │ quality-check   │  tap       │ quality-check   │
 │ ama             │  ───▶      │ v1.0            │
 │ audit-debt      │            │ Description     │
 │                 │            │ [Run] [History] │
 │                 │  browser   │ Scope input     │
 │                 │  back ◀─── │ [Run ▶]         │
 └──────────────────┘  OR btn   └──────────────────┘
  LIST                           DETAIL
                            ✓ Back button (mobile only)
                            ✓ Browser back works
                            ✓ URL is /tools/quality-check
```

---

### Code delta — what actually changes

**App.tsx** (+1 line):
```tsx
<Route path="/tools" element={<ToolsPage />} />
<Route path="/tools/:name" element={<ToolsPage />} />  // ← add
```

**ToolsPage.tsx** — state → URL param:
```tsx
// REMOVE (3 lines):
const [selectedName, setSelectedName] = useState<string | null>(null)
const selectedNameRef = useRef(selectedName)
selectedNameRef.current = selectedName
// REMOVE (2 lines):
} else if (data.length > 0 && !selectedNameRef.current) {
  setSelectedName(data[0].name)
}

// ADD:
const { name } = useParams<{ name?: string }>()
const navigate = useNavigate()
const selectedTool = tools.find(t => t.name === name) ?? null
const showMobileDetail = !!name
```

**ToolsPage.tsx** — left panel (2 changes):
```tsx
// w-64 → w-72 (line 1254)
// selectedTool → showMobileDetail (lines 1255, 1294)
```

**ToolsPage.tsx** — tool selection:
```tsx
// setSelectedName(tool.name) → navigate(`/tools/${tool.name}`)
// setSelectedName(null) → navigate('/tools')
```

**ToolsPage.tsx** — pass onBack:
```tsx
<ToolRunPanel
  tool={selectedTool}
  onReload={(n) => { setLoading(true); load(n) }}
  onBack={() => navigate('/tools')}  // ← add
/>
```

**ToolRunPanel** — add back button:
```tsx
interface ToolRunPanelProps {
  tool: ToolMeta
  onReload: (name?: string) => void
  onBack?: () => void  // ← add
}

// In header (before tool name, mobile-only):
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

**Blast radius:** ~55 lines changed. Two files: `ToolsPage.tsx` + `App.tsx`.

---

### After: ToolsPage parity with PonderPage

| Dimension           | PonderPage       | ToolsPage after   |
|---------------------|------------------|-------------------|
| Left panel width    | `w-72` (288px)   | `w-72` (288px) ✓  |
| URL routing         | `/ponder/:slug`  | `/tools/:name` ✓  |
| Mobile back nav     | `navigate('/ponder')` | `navigate('/tools')` ✓ |
| Browser back button | ✓ works          | ✓ works ✓         |
| Auto-select on load | No               | No (removed) ✓    |
| showMobileDetail    | `!!slug`         | `!!name` ✓        |
| Empty state         | Shows full list  | Shows full list ✓ |

---

⚑ **Decided:** PonderPage unchanged — already correct.
⚑ **Decided:** No shared MasterDetailLayout component — convention alignment is sufficient.
⚑ **Decided:** Design is complete and matches Session 2 architecture decision exactly.

?  **Open:** Does Jordan want to proceed? If yes → create feature and implement.
?  **Open:** Confirm empty-state UX: no auto-select means landing on /tools shows list, nothing highlighted. Same as /ponder. Is that correct?
