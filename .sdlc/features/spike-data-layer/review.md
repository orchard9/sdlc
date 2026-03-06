# Code Review: spike-data-layer

## Files changed

- `crates/sdlc-core/src/paths.rs` — added `SPIKES_DIR` constant + `spike_dir`, `spike_findings_path`, `spike_state_path` helpers
- `crates/sdlc-core/src/spikes.rs` — new module: 20 tests, ~350 lines
- `crates/sdlc-core/src/lib.rs` — added `pub mod spikes`

## Findings

### Correctness

- Parsing is purely line-by-line with no regex; handles all four verdict states and missing sections correctly — verified by 20 unit tests
- `list()` sorts by date descending; undated entries sort to end
- `store_in_knowledge` idempotency confirmed by test: second call returns same slug, no duplicate entry
- `promote_to_ponder` uses existing `ponder::PonderEntry::create` and `ponder::capture_content` — no new I/O primitives
- State file writes use `crate::io::atomic_write` — consistent with project conventions

### Code quality

- No `unwrap()` or `expect()` outside test helpers — ✓
- All public functions return `Result<_>` — ✓
- `SpikeState` uses `#[serde(default)]` implicitly via `derive(Default)` — absent state.yaml deserializes gracefully — ✓
- Side-effect error in `list()` (auto-filing REJECT) swallowed with `let _ = ...` — acceptable for background behaviour, documented in doc comment

### Convention compliance

- Follows same module structure as `investigation.rs`, `ponder.rs`: enums → types → internal helpers → public API → tests — ✓
- `SdlcError::Io` used for "not found" case (no domain-specific `SpikeNotFound` variant needed since the feature is new and no callers yet expect it)

### Minor notes

- `ParsedFindings` is `pub(super)` only through `parse_findings` (private) — correctly unexported
- Knowledge code "900" is hardcoded. If catalog codes change this will need updating — tracked as acceptable for now; knowledge categorisation is agent-owned, not machine-enforced

## Verdict: APPROVED

All tasks implemented. 20/20 tests pass. Zero clippy warnings.
