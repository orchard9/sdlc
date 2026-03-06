# Tasks: spike-data-layer

## Implementation tasks (from T1–T7 in feature tracker)

1. Add spike path helpers to `crates/sdlc-core/src/paths.rs`: `spike_dir`, `spike_findings_path`, `spike_state_path`, `SPIKES_DIR` constant
2. Create `crates/sdlc-core/src/spikes.rs`: `SpikeVerdict` enum, `SpikeState` (serde), `SpikeEntry` struct
3. Implement `parse_findings(content: &str)` — pure, line-by-line, no regex
4. Implement `list(root)`, `load(root, slug)`, `extract_open_questions(findings)`
5. Implement `promote_to_ponder(root, slug, override)` integrating ponder::create + capture_artifact + state write
6. Implement `store_in_knowledge(root, slug)` integrating knowledge::create + append_content + update + state write
7. Register `pub mod spikes` in `crates/sdlc-core/src/lib.rs`
8. Unit tests: parse all verdicts, missing sections, list sort, promote artifacts, store_in_knowledge idempotency

## QA

- `SDLC_NO_NPM=1 cargo test --all` passes
- `cargo clippy --all -- -D warnings` passes
