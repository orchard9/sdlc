# v13-backlog Acceptance Test

## Scenario: Agent captures concern immediately upon discovery

- [ ] Run `sdlc backlog add "AuthMiddleware in auth.rs: token validation has a race condition under concurrent requests" --kind concern --evidence "crates/sdlc-server/src/auth.rs:42" --source-feature server-remote-state`
- [ ] Output includes: item ID (B-prefixed), title echo, and source_feature confirmation
- [ ] Verify item in `sdlc backlog list` with status `open`, kind `concern`, id `B1`
- [ ] Run `sdlc backlog show B1` — confirm title, kind, evidence, source_feature, created_at

## Scenario: kind and evidence fields are present and useful

- [ ] `sdlc backlog add "Consider extracting redb retry logic into a shared utility" --kind idea --source-feature run-events-api`
- [ ] `sdlc backlog add "Middleware session store grows unbounded without eviction" --kind debt --evidence "crates/sdlc-server/src/routes/runs.rs" --source-feature backlog-server`
- [ ] `sdlc backlog list` shows all 3 items with kind column visible
- [ ] `sdlc backlog list --source-feature run-events-api` shows only the idea item

## Scenario: Auto-infer source_feature

- [ ] Call `sdlc backlog add "Something noticed" --kind concern` (no --source-feature)
- [ ] Output includes warning "No source feature provided — inferred from state.yaml: <slug>" or "No source feature found — item recorded without attribution"
- [ ] Item recorded either way; capture is never blocked

## Scenario: Park with required reason

- [ ] Run `sdlc backlog park B2` (no --reason) — command should reject with helpful message
- [ ] Run `sdlc backlog park B2 --reason "Revisit after v14 ships"` — succeeds
- [ ] `sdlc backlog list` (default) shows only open items (B2 not shown)
- [ ] `sdlc backlog list --all` shows B2 with status `parked` and park_reason visible

## Scenario: Promote to a feature in a specific milestone

- [ ] Run `sdlc backlog promote B1 --slug auth-race-fix --milestone v13-backlog`
- [ ] Output shows: "Promoted B1 → feature auth-race-fix (added to milestone v13-backlog)"
- [ ] `sdlc backlog show B1` shows `status: promoted`, `promoted_to: auth-race-fix`
- [ ] `sdlc feature list` includes `auth-race-fix` in draft phase
- [ ] `sdlc milestone info v13-backlog` includes `auth-race-fix` in feature list

## Scenario: REST API

- [ ] `GET /api/backlog` returns all items as JSON
- [ ] `GET /api/backlog?status=open` returns only open items
- [ ] `GET /api/backlog?source_feature=run-events-api` returns only items from that feature
- [ ] `POST /api/backlog` with `{"title": "Test concern", "kind": "concern"}` creates item with ID
- [ ] `POST /api/backlog/B2/park` with `{"park_reason": "low priority"}` succeeds
- [ ] `POST /api/backlog/B1/promote` with `{"slug": "new-feature", "milestone_slug": "v13-backlog"}` creates feature and updates item

## Scenario: Dashboard legibility (first-time user)

- [ ] Empty state shows 3-sentence explanation: origin, meaning, non-blocking nature
- [ ] Each item card shows: title, kind badge (concern/idea/debt), source_feature link, timestamp
- [ ] Promote button has tooltip: "Creates a draft feature. No work begins automatically."
- [ ] Clicking Park opens a modal that requires a reason before confirming
- [ ] Source_feature filter dropdown lets operator triage by feature origin
- [ ] Open items older than 60 days show a staleness indicator
- [ ] After promotion: toast appears with link to new feature card

## Scenario: Guidance enforces immediate capture

- [ ] sdlc-run command template includes instruction: "Capture concerns IMMEDIATELY upon discovery, before the next task"
- [ ] sdlc-run template includes example of a well-formed title with component reference
- [ ] .sdlc/guidance.md §12 Session Close Protocol is present
- [ ] §12 includes vocabulary table: backlog items vs tasks vs features
