# QA Plan: Park/Unpark CLI Commands and REST Endpoints

## TC-1: compute_status returns Parked when parked_at is set
- Create milestone, call `park()`, verify `compute_status` returns `Parked`
- Verify priority: even with all features released, Parked wins over Verifying

## TC-2: compute_status priority ordering
- Skipped + Parked: Skipped wins
- Released + Parked: Released wins
- Parked alone: Parked
- Parked + all features released: Parked (not Verifying)

## TC-3: Unpark clears parked_at and restores computed status
- Park then unpark, verify `parked_at` is None, status returns to Active/Verifying as appropriate

## TC-4: CLI park/unpark commands
- `sdlc milestone park <slug>` — verify parked_at is set in milestone YAML
- `sdlc milestone unpark <slug>` — verify parked_at is cleared
- JSON output mode: verify correct status field

## TC-5: REST park/unpark endpoints
- `POST /api/milestones/:slug/park` — verify 200 response with status "parked"
- `POST /api/milestones/:slug/unpark` — verify 200 response with computed status
- `GET /api/milestones/:slug` — verify parked_at appears in response

## TC-6: Parallel work excludes parked milestones
- Create milestone with features, park it, verify `select_parallel_work` skips it
- Unpark it, verify it appears in parallel work again

## TC-7: Backward compatibility
- Load a milestone YAML file that has no `parked_at` field — verify it deserializes to `parked_at: None` and status is Active

## TC-8: Idempotency
- Park an already-parked milestone — no error
- Unpark an already-unparked milestone — no error

## Test Execution

- TC-1 through TC-3, TC-6 through TC-8: unit tests in `milestone.rs` and `parallel_work.rs`
- TC-4: `SDLC_NO_NPM=1 cargo test --all` (integration tests)
- TC-5: integration tests in `sdlc-server/tests/integration.rs`
