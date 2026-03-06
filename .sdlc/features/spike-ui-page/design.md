# Spike UI Page — Design

## Architecture

`SpikePage` follows the same two-pane split pattern used by `InvestigationPage`, `EvolvePage`, and `GuidelinePage`:

- **Left pane** (fixed width, `w-72`): filterable list of spikes with verdict badges.
- **Right pane** (flex-1): detail view showing verdict-specific content and actions.
- **Mobile**: single-pane — list or detail based on route param presence.

[Mockup](mockup.html)

## Component Tree

```
SpikePage (pages/SpikePage.tsx)
├── SpikeList (left pane)
│   ├── VerdictFilter tabs (ALL / ADOPT / ADAPT / REJECT)
│   └── SpikeRow[] — title, verdict badge, date, question snippet
│       ├── ADOPT: "Next: /sdlc-hypothetical-planning" chip
│       ├── ADAPT: "Promote to Ponder →" button (or ponder link if promoted)
│       └── REJECT: "Stored in Knowledge" badge (linked if knowledge_slug set)
└── SpikeDetailPane (right pane)
    ├── Breadcrumb + title + verdict badge
    ├── The question (prominent)
    ├── Date
    ├── ADOPT: WhatsNextSection (hint + copy button for CLI command)
    ├── ADAPT: PromoteButton (calls POST /api/spikes/:slug/promote → navigate)
    ├── REJECT: KnowledgeSection (badge + link)
    └── PonderLineage (shown when ponder_slug set)
```

## Data Flow

```
GET /api/spikes           → SpikeList (all spikes)
GET /api/spikes/:slug     → SpikeDetailPane (single spike)
POST /api/spikes/:slug/promote → returns { ponder_slug } → navigate to /ponder/:ponder_slug
```

No SSE subscription needed — spike data is immutable after creation.

## Types (frontend/src/lib/types.ts additions)

```typescript
export type SpikeVerdict = 'ADOPT' | 'ADAPT' | 'REJECT'

export interface SpikeSummary {
  slug: string
  title: string
  verdict: SpikeVerdict
  date: string
  the_question: string
  ponder_slug?: string
  knowledge_slug?: string
}

export type SpikeDetail = SpikeSummary  // same shape, detail endpoint returns same fields
```

## API Client (frontend/src/api/client.ts additions)

```typescript
getSpikes: () => request<SpikeSummary[]>('/api/spikes'),
getSpike: (slug: string) => request<SpikeDetail>(`/api/spikes/${slug}`),
promoteSpike: (slug: string) => request<{ ponder_slug: string }>(`/api/spikes/${slug}/promote`, { method: 'POST' }),
```

## Sidebar (frontend/src/components/layout/Sidebar.tsx)

Add to `plan` group, after `guidelines`:
```typescript
{ path: '/spikes', label: 'Spikes', icon: FlaskConical, exact: false },
```

Import `FlaskConical` from `lucide-react` (already available in the package).

## BottomTabBar (frontend/src/components/layout/BottomTabBar.tsx)

Add `/spikes` to the `Plan` tab roots array.

## Router (frontend/src/App.tsx)

```tsx
import { SpikePage } from '@/pages/SpikePage'
...
<Route path="/spikes" element={<SpikePage />} />
<Route path="/spikes/:slug" element={<SpikePage />} />
```

## Verdict Color Scheme

| Verdict | Badge background | Badge text | Tailwind classes |
|---------|-----------------|------------|-----------------|
| ADOPT | green-100 / green-900 dark | green-700 / green-300 dark | `bg-green-100 text-green-700 dark:bg-green-900/30 dark:text-green-300` |
| ADAPT | yellow-100 / yellow-900 dark | yellow-700 / yellow-300 dark | `bg-yellow-100 text-yellow-700 dark:bg-yellow-900/30 dark:text-yellow-300` |
| REJECT | red-100 / red-900 dark | red-700 / red-300 dark | `bg-red-100 text-red-700 dark:bg-red-900/30 dark:text-red-300` |

## Empty State

When `GET /api/spikes` returns an empty array:
- Icon: `FlaskConical` (muted, large)
- Heading: "No spikes yet"
- Body: "Spikes are time-boxed investigations that answer one question. When a spike concludes, it gets an ADOPT, ADAPT, or REJECT verdict — and that verdict drives what happens next."
- CLI example code block: `/sdlc-spike <slug> — <the question to answer>`
- CTA: "Run `/sdlc-spike` from your AI coding CLI to create one."

## Error States

- List fetch fails: show inline error message (no crash).
- Detail fetch fails or slug not found: show "Spike not found" placeholder in right pane.
- Promote fails: show inline error message near the button (do not navigate).
