# QA Plan: Blocked Feature UX — BlockedPanel

## Scope

Verify the backend `remove_blocker` logic, the DELETE API route, and the frontend
`BlockedPanel` component with its integration in `FeatureDetail`.

---

## Backend unit tests (Rust)

### BQ1 — `remove_blocker` removes the correct element

```
Given: Feature with blockers = ["a", "b", "c"]
When:  remove_blocker(1)
Then:  blockers == ["a", "c"], updated_at is refreshed
```

### BQ2 — `remove_blocker` out-of-range returns Err

```
Given: Feature with blockers = ["a"]
When:  remove_blocker(1)
Then:  Err(SdlcError::InvalidInput(...))
```

### BQ3 — `remove_blocker` on empty list returns Err

```
Given: Feature with blockers = []
When:  remove_blocker(0)
Then:  Err(...)
```

---

## API integration tests

### AQ1 — DELETE removes blocker and saves

```
Given: Feature "f1" with blockers = ["x", "y"]
When:  DELETE /api/features/f1/blockers/0
Then:  200 { "ok": true }
       Feature.load("f1").blockers == ["y"]
```

### AQ2 — DELETE with reason stores decision comment

```
Given: Feature "f1" with blockers = ["x"]
When:  DELETE /api/features/f1/blockers/0  { "reason": "no longer needed" }
Then:  200 { "ok": true }
       Feature.load("f1").blockers == []
       Feature.load("f1").comments contains comment with body containing "no longer needed"
       and flag == "decision"
```

### AQ3 — DELETE with out-of-range idx returns 4xx

```
Given: Feature "f1" with blockers = ["x"]
When:  DELETE /api/features/f1/blockers/5
Then:  4xx error response
```

### AQ4 — DELETE on non-existent feature returns 404

```
Given: No feature "doesnotexist"
When:  DELETE /api/features/doesnotexist/blockers/0
Then:  404
```

---

## Frontend component tests (manual / visual)

### FQ1 — BlockedPanel is hidden when feature is not blocked

```
Given: Feature with blocked = false
Then:  BlockedPanel is not rendered in FeatureDetail
```

### FQ2 — BlockedPanel shows blocker list when feature is blocked

```
Given: Feature with blockers = ["dep-a", "something else"]
Then:  Both blockers are visible in the panel
       "dep-a" shows a [→ dep-a] link if dep-a is in allSlugs
       "something else" shows no link
```

### FQ3 — Remove button reveals inline reason UI

```
Given: BlockedPanel rendered with one blocker
When:  User clicks [Remove]
Then:  Reason input, [Confirm], and [Cancel] are shown inline under the blocker row
```

### FQ4 — Confirm without reason removes blocker

```
Given: BlockedPanel with one blocker, reason input empty
When:  User clicks [Remove] then [Confirm]
Then:  DELETE /api/features/:slug/blockers/0 is called with no reason body
       On success, panel reflects updated blocker list via SSE refresh
```

### FQ5 — Confirm with reason removes blocker and stores comment

```
Given: BlockedPanel with one blocker
When:  User fills reason "skipping dependency", clicks [Confirm]
Then:  DELETE /api/features/:slug/blockers/0 called with { reason: "skipping dependency" }
       Decision comment created on feature
```

### FQ6 — Cancel hides inline remove UI

```
Given: Inline remove UI visible for blocker 0
When:  User clicks [Cancel]
Then:  Reason input hidden, blocker row returns to default state
```

### FQ7 — "Run with direction" disabled when input empty

```
Given: BlockedPanel rendered
When:  direction input is empty
Then:  "Run with direction" button is disabled
```

### FQ8 — "Run with direction" calls onRunWithDirection with input value

```
Given: Direction input contains "skip dep, use stub"
When:  User clicks [Run with direction]
Then:  onRunWithDirection called with "skip dep, use stub"
       Agent run starts via AgentRunContext
```

### FQ9 — "Run with direction" disabled when run is already in progress

```
Given: isRunning = true
Then:  "Run with direction" button is disabled
```

---

## Build / lint

### BL1 — Rust build and tests pass

```
SDLC_NO_NPM=1 cargo test --all
cargo clippy --all -- -D warnings
```

Expected: 0 errors, 0 warnings.

### BL2 — Frontend type-checks

```
cd frontend && npm run build
```

Expected: clean build, no TypeScript errors.
