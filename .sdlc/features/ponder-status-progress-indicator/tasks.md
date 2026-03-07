# Tasks: Ponder Status Step Indicator

## T1: Create PonderStepIndicator component
- [ ] Create `frontend/src/components/ponder/PonderStepIndicator.tsx`
- [ ] Implement full variant: three labeled steps with check marks, current highlight, upcoming dim
- [ ] Implement compact variant: three dots with connecting lines
- [ ] Handle parked status: all muted + parked badge/label
- [ ] Use status-specific colors: violet (exploring), amber (converging), emerald (committed)

## T2: Integrate into PonderPage detail view
- [ ] Add `PonderStepIndicator` to the entry detail panel in `PonderPage.tsx`
- [ ] Replace or augment the existing status badge area with the step indicator
- [ ] Verify all four states render correctly in the detail view

## T3: Integrate compact variant into list rows
- [ ] Add `<PonderStepIndicator compact />` to list rows in `PonderPage.tsx`
- [ ] Remove redundant StatusBadge from list rows if the compact indicator replaces it
- [ ] Verify list row layout with compact indicator across all states
