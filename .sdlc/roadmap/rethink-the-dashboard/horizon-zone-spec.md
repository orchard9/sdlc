# Horizon Zone — Design Spec

## Purpose
The single forward-looking surface on the dashboard. Shows everything from "concrete and ready to run" down to "idea being explored."

## Three subgroups (in visual order)

### 1. Queued Milestones (concrete)
- Planned milestones with defined slugs and features
- Solid card, milestone icon, feature count badge
- Label: "Queued"

### 2. Ready to Commit (amber)
- Ponders with status: `converging`
- Amber dashed border, lightbulb icon
- CTA: [Commit ->] — routes to /sdlc-ponder-commit flow
- These are "almost milestones" — one decision away from real work

### 3. Exploring (muted)
- Ponders with status: `exploring`
- Ghost/muted appearance, lightbulb icon
- Label: "Exploring", CTA: [Continue ->]
- Show max 3; truncate with "View all ->" to /ponder

## Exclusions
- Parked ponders (`status: parked`) are NOT shown in Horizon

## Section header
```
HORIZON                              [+ New idea]
N planned · M pondering
```
- [+ New idea] button routes to /ponder new entry dialog
- Count label replaces the standalone Ideation Strip from prior design

## Eliminated component
The standalone **Ideation Strip** ("N pondering · M converging — View workspace ->") is removed.
Its function is absorbed into the Horizon header.