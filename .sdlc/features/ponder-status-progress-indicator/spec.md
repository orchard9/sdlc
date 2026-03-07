# Spec: Ponder Status Step Indicator

## Problem

Ponder entries progress through a lifecycle: **exploring → converging → committed** (with **parked** as a side-exit). Currently the UI shows status as a flat badge — there's no visual indication of where an entry sits in its progression or what steps remain. Users scanning the roadmap page can't quickly gauge momentum.

## Solution

Add a step indicator component that renders the ponder lifecycle as a horizontal sequence of steps, showing completed, current, and upcoming states. Display it in the ponder entry detail view (replacing or augmenting the current status badge area) and optionally as a compact variant in list rows.

## Requirements

### R1: PonderStepIndicator component
- Render three primary steps: **Exploring → Converging → Committed**
- Each step shows: label, icon/check for completed, highlight for current
- Completed steps show a check mark and muted styling
- Current step is highlighted with its status color (violet=exploring, amber=converging, emerald=committed)
- Future steps show muted/disabled styling
- **Parked** status renders all steps muted with a "Parked" overlay or badge

### R2: Detail view integration
- Display PonderStepIndicator in `PonderPage.tsx` entry detail panel, above or replacing the current status badge row
- The indicator is the primary way users see progression state

### R3: List row compact variant
- Optional compact/mini variant for the list row (dots or small circles instead of full labels)
- Falls back to existing StatusBadge if space is constrained

### R4: Data requirements
- No new backend fields needed — derive step from existing `PonderStatus` enum
- Mapping: `exploring` → step 1, `converging` → step 2, `committed` → step 3, `parked` → special state

## Out of Scope
- Automatic status advancement logic (status transitions remain manual/agent-driven)
- Backend changes to the PonderStatus enum
- Changes to the status change modal

## Reference
- Existing `PhaseStrip` component (`frontend/src/components/investigation/PhaseStrip.tsx`) provides a proven pattern for step sequences with completed/current/upcoming styling
- `StatusBadge` color scheme: violet (exploring), amber (converging), emerald (committed), neutral (parked)
