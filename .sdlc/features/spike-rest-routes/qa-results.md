# QA Results: Spike REST Routes

## Test Execution

Command: `SDLC_NO_NPM=1 cargo test --package sdlc-server -- spikes`

## Results

```
running 10 tests
test routes::spikes::tests::list_spikes_empty_when_no_dir ... ok
test routes::spikes::tests::list_spikes_returns_entry ... ok
test routes::spikes::tests::promote_returns_404_for_unknown_spike ... ok
test routes::spikes::tests::get_spike_returns_404_for_unknown ... ok
test routes::spikes::tests::promote_returns_422_for_adopt_verdict ... ok
test routes::spikes::tests::promote_returns_422_for_reject_verdict ... ok
test routes::spikes::tests::promote_no_verdict_returns_422 ... ok
test routes::spikes::tests::get_spike_returns_findings_content ... ok
test routes::spikes::tests::promote_adapt_spike_returns_ponder_slug ... ok
test routes::spikes::tests::promote_with_ponder_slug_override ... ok

test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 185 filtered out; finished in 0.56s
```

Full suite: `SDLC_NO_NPM=1 cargo test --all` — all test suites passed, 0 failures.

Clippy: `cargo clippy --all -- -D warnings` — no warnings.

## Coverage Against QA Plan

| Scenario | Result |
|---|---|
| GET /api/spikes — no dir | PASS |
| GET /api/spikes — returns entry with correct verdict/date | PASS |
| GET /api/spikes/:slug — 404 for unknown | PASS |
| GET /api/spikes/:slug — returns findings content | PASS |
| POST /api/spikes/:slug/promote — 404 for unknown | PASS |
| POST /api/spikes/:slug/promote — 422 for ADOPT | PASS |
| POST /api/spikes/:slug/promote — 422 for REJECT | PASS |
| POST /api/spikes/:slug/promote — 422 for no verdict | PASS |
| POST /api/spikes/:slug/promote — 200 + ponder_slug for ADAPT | PASS |
| POST /api/spikes/:slug/promote — override ponder_slug | PASS |

## Status: PASSED
