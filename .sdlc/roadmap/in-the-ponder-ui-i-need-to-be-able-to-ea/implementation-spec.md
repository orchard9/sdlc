# User Message Navigation — Implementation Spec

## Problem
Owner messages in the ponder UI are (a) invisible in completed sessions due to a bug, and (b) have no navigation mechanism.

## Root Causes
1. `ownerName` only passed to `SessionBlock` during active runs → owner messages undetectable in history (visual bug)
2. `owner_name` not persisted in manifest → can't reconstruct from historical sessions

## Solution

### Rust / API changes
- Add `owner_name: Option<String>` to `PonderEntry` in `crates/sdlc-core/src/ponder.rs`
- Set on first ponder chat start in `crates/sdlc-server/src/routes/runs.rs`  
- Expose in `GET /api/roadmap/:slug` response → `PonderDetail`

### Frontend changes
- Add `owner_name: string | null` to `PonderDetail` type in `frontend/src/lib/types.ts`
- In `DialoguePanel`, derive ownerName: `entry.owner_name ?? (runState.status === 'running' ? runState.ownerName : null)`
- Pass `ownerName` to **all** `SessionBlock` instances (not gated on `isRunning`)
- Add `data-owner-msg` attribute to rendered `PartnerMessage` when `isOwner=true`
- Add `OwnerMessageNav` component — floating pill, absolute-positioned within scroll container
  - Collects owner message elements via `querySelectorAll('[data-owner-msg]')` on scroll ref
  - Tracks current index, scrolls on arrow press with `scrollIntoView({ behavior: 'smooth', block: 'center' })`
  - Only renders when count ≥ 2

## UI Sketch
```
┌──────────────────────────────────────────────────────────────────┐
│  scroll stream                                                   │
│                                                                  │
│  ┌─────────────────────────┐   ← owner message (highlighted)   │
│  │ Jordan · Owner          │                                     │
│  │ in the ponder ui I need │                                     │
│  └─────────────────────────┘                                     │
│                                                                  │
│  [agent content, partner messages...]                            │
│                                                                  │
│                               ┌─────────────────┐               │
│                               │  ↑  2 / 5  ↓   │  ← float     │
│                               └─────────────────┘               │
└──────────────────────────────────────────────────────────────────┘
```

## Acceptance Criteria
- Owner messages in all past sessions have the border+bg visual distinction
- Floating nav pill appears when ≥ 2 owner messages exist
- Prev/next arrows scroll smoothly to each owner message
- Current position counter is accurate ("2 / 5")
- Nav does not render for ponies with 0 or 1 owner messages

## Out of Scope (v1)
- Filter/reading mode (show only owner messages)
- Sidebar jump list
- Keyboard nav shortcuts
