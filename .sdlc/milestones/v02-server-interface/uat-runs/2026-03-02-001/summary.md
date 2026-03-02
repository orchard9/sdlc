# UAT Run — Server as remote PM interface
**Date:** 2026-03-02T03:45:00Z
**Verdict:** Pass
**Tests:** 10/10
**Tasks created:** none

## Build & Test Results

| Check | Result |
|---|---|
| `SDLC_NO_NPM=1 cargo build --all` | PASS — no errors |
| `cargo clippy --all -- -D warnings` | PASS — no warnings |
| `SDLC_NO_NPM=1 cargo test -p sdlc-server` | PASS — 92 unit + 25 integration tests |

## Smoke Test Results

| # | Test | Result |
|---|---|---|
| 1 | `GET /api/features/:slug/directive` returns full Classification JSON | PASS |
| 2 | `/directive` output matches `sdlc next --for <slug> --json` exactly | PASS |
| 3 | `GET /api/features/nonexistent/directive` returns 404 | PASS |
| 4 | `POST /api/artifacts/:slug/spec/draft` returns `{slug, artifact_type, status: draft}` | PASS |
| 5 | `POST /api/artifacts/nonexistent/spec/draft` returns 404 | PASS |
| 6 | `POST /api/artifacts/:slug/bogus/draft` returns 404 for invalid type | PASS |
| 7 | `POST /api/features/:slug/merge` returns 400 when not in merge phase | PASS |
| 8 | `POST /api/features/nonexistent/merge` returns 404 | PASS |
| 9 | `POST /api/features/:slug/merge` returns `{slug, phase: released, merged: true}` | PASS |
| 10 | `GET /api/features/:slug` confirms `phase: released` after merge | PASS |

## Note on Binary Version

The `sdlc ui start` command was using a stale installed binary (March 1 build).
Updated via `cargo install --path crates/sdlc-cli --force` before running smoke tests.
All routes were confirmed registered and functional in the latest binary.
