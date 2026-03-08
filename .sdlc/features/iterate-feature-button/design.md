# Design: Iterate Button on FeatureDetail

## Component Changes

### FeatureDetail.tsx

Add an "Iterate" button to the existing released banner (the `classification.action === 'done'` block, lines ~221-243).

#### Button Placement

Inside the released banner, add a right-aligned button in the header row next to the "Released" label:

```tsx
<div className="flex items-center justify-between">
  <div className="flex items-center gap-2">
    <CheckCircle2 className="w-4 h-4 text-green-400" />
    <span className="text-sm font-medium text-green-400">Released</span>
  </div>
  <button onClick={handleIterate} disabled={iterating} ...>
    {iterating ? <Loader2 spinning /> : <RefreshCw />}
    Iterate
  </button>
</div>
```

#### Handler Logic

```typescript
const [iterating, setIterating] = useState(false)

const handleIterate = async () => {
  setIterating(true)
  try {
    const newSlug = nextIterationSlug(feature.slug, existingSlugs)
    await api.createPonderEntry({
      slug: newSlug,
      title: `Iterate: ${feature.title}`,
      brief: `Follow-up iteration on "${feature.title}". Original feature slug: ${feature.slug}.`,
    })
    navigate(`/ponder/${newSlug}`)
  } catch (err) {
    console.error('Failed to create iteration ponder:', err)
    // Could add toast notification in the future
  } finally {
    setIterating(false)
  }
}
```

#### Slug Collision Avoidance

The `nextIterationSlug` utility (from `iterate-slug-utility`) handles version incrementing and collision checks. It needs a list of existing ponder slugs to avoid collisions. We fetch the roadmap list on mount (similar to how `allSlugs` is fetched for features).

```typescript
const [ponderSlugs, setPonderSlugs] = useState<string[]>([])

useEffect(() => {
  api.getRoadmap(true).then(entries =>
    setPonderSlugs(entries.map(e => e.slug))
  ).catch(() => {})
}, [])
```

## New Imports

- `RefreshCw` from `lucide-react` (iterate icon)
- `useNavigate` from `react-router-dom`
- `nextIterationSlug` from `@/lib/iterateSlug` (provided by iterate-slug-utility feature)

## Data Flow

```
User clicks "Iterate"
  → nextIterationSlug(feature.slug, ponderSlugs)
  → api.createPonderEntry({ slug, title, brief })
  → navigate(`/ponder/${newSlug}`)
```

## No Backend Changes

Uses existing `POST /api/roadmap` endpoint which accepts `{ slug, title, brief }`.

## Visual Style

The button uses the same muted style as other secondary actions in the released banner — small, understated, not the primary action. Uses `bg-green-500/20 text-green-400 hover:bg-green-500/30` to match the banner's green theme.
