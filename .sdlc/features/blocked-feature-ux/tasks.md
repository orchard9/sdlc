# Tasks: Blocked Feature UX — BlockedPanel

## T1: Add `remove_blocker` method to `Feature` in `sdlc-core`

**File:** `crates/sdlc-core/src/feature.rs`

Add a public method that removes a blocker by index:

```rust
pub fn remove_blocker(&mut self, idx: usize) -> Result<()> {
    if idx >= self.blockers.len() {
        return Err(SdlcError::InvalidInput(format!(
            "blocker index {} out of range (len={})",
            idx,
            self.blockers.len()
        )));
    }
    self.blockers.remove(idx);
    self.updated_at = Utc::now();
    Ok(())
}
```

Add a unit test:
- Given a feature with two blockers, removing index 0 leaves only the second blocker.
- Removing an out-of-range index returns `Err`.

**Estimate:** ~20 lines

---

## T2: Add `DELETE /api/features/:slug/blockers/:idx` server route

**Files:**
- `crates/sdlc-server/src/routes/features.rs` — add handler
- `crates/sdlc-server/src/lib.rs` — register route

Handler:
1. Load feature.
2. Call `feature.remove_blocker(idx)`.
3. If `body.reason` is non-empty, append a `decision` comment with the text "Blocker removed: {reason}".
4. Save feature.
5. Return `{ "ok": true }`.

The `RemoveBlockerBody` struct deserializes an optional `reason: Option<String>`.
The route path is `/api/features/{slug}/blockers/{idx}` using `delete()`.

**Estimate:** ~35 lines

---

## T3: Create `BlockedPanel.tsx` component

**File:** `frontend/src/components/features/BlockedPanel.tsx`

New React component:

Props:
```ts
interface BlockedPanelProps {
  slug: string
  blockers: string[]
  allSlugs: string[]
  isRunning: boolean
  onRunWithDirection: (direction: string) => void
}
```

State:
- `removingIdx: number | null` — active inline remove UI
- `reasons: Record<number, string>` — per-blocker reason text
- `direction: string` — direction input

Behavior:
- Renders amber-tinted panel with `AlertTriangle` icon and "Blocked" heading.
- Lists blockers. If a blocker text exactly matches a slug in `allSlugs`, render a
  `<Link to={/features/${blocker}}>→ {blocker}</Link>` after the text.
- Per-blocker: `[Remove]` button; on click, reveals reason input + `[Confirm]` + `[Cancel]`.
- Confirm: calls `DELETE /api/features/:slug/blockers/:idx` with optional `{ reason }`.
  Uses `fetch`. On success, resets UI state (SSE Update will refresh the parent).
- Direction section (below divider): controlled text input bound to `direction`.
- "Run with direction" button: calls `onRunWithDirection(direction)`, disabled when
  `direction.trim() === ''` or `isRunning`.

**Estimate:** ~90 lines

---

## T4: Integrate `BlockedPanel` into `FeatureDetail`

**File:** `frontend/src/pages/FeatureDetail.tsx`

Changes:
1. Import `BlockedPanel`.
2. Fetch or derive `allSlugs` (use the existing features list from `useFeatures` hook or
   from a `GET /api/features` call; if `useFeatures` is not available in this page,
   inline a small `useSlugs` fetch).
3. In the JSX, after the feature header block and before the "Next action" card, add:
   ```tsx
   {feature.blocked && (
     <BlockedPanel
       slug={slug}
       blockers={feature.blockers}
       allSlugs={allSlugs}
       isRunning={running}
       onRunWithDirection={(direction) => {
         startRun({
           key: slug,
           runType: 'feature',
           target: slug,
           label: slug,
           startUrl: `/api/run/${slug}`,
           stopUrl: `/api/run/${slug}/stop`,
           context: direction,
         })
       }}
     />
   )}
   ```
4. Verify that `startRun` in `AgentRunContext` supports the `context` field (check the
   existing interface). If not yet supported, add it.

**Estimate:** ~15 lines

---

## Task summary

| ID | Title | Est. lines |
|---|---|---|
| T1 | `remove_blocker` in sdlc-core | ~20 |
| T2 | DELETE route in sdlc-server | ~35 |
| T3 | `BlockedPanel.tsx` new component | ~90 |
| T4 | Integrate into FeatureDetail | ~15 |

Total: ~160 lines (slightly above initial estimate due to test code).
