# QA Plan: Fix Missing `_shared/agent.ts` in Tool Installation

## Scenarios

### S1: Build compiles with new constant

- Run `SDLC_NO_NPM=1 cargo build --all`
- Expected: zero errors; `include_str!` resolves to `.sdlc/tools/_shared/agent.ts`

### S2: Test suite passes

- Run `SDLC_NO_NPM=1 cargo test --all`
- Expected: all tests pass, no regressions in init or tool tests

### S3: `agent.ts` is written by init

- In a temp directory, run `sdlc init` (or verify via init integration tests)
- Expected: `.sdlc/tools/_shared/agent.ts` exists and matches source

### S4: Existing shared files still installed

- After init, verify `_shared/types.ts`, `_shared/log.ts`, `_shared/config.ts`, `_shared/runtime.ts` all exist
- Expected: no regressions in other shared file installation

### S5: `cargo clippy` clean

- Run `cargo clippy --all -- -D warnings`
- Expected: no new warnings
