# QA Plan: Fix Investigation Session Completion

## Test Cases

### T1: CLI --output-type flag
1. Create a root-cause investigation
2. Run `sdlc investigate update <slug> --output-type task` -- should succeed
3. Verify manifest has `output_type: task`
4. Create an evolve investigation
5. Run `sdlc investigate update <slug> --output-type task` -- should fail (root-cause only)

### T2: CLI --output-ref for evolve
1. Create an evolve investigation
2. Run `sdlc investigate update <slug> --output-ref my-feature` -- should succeed
3. Verify manifest has `output_refs: [my-feature]`
4. Run again with `--output-ref another-feature` -- should append
5. Verify manifest has `output_refs: [my-feature, another-feature]`

### T3: REST API evolve output fields
1. Create an evolve investigation via REST
2. PUT with `{ "output_type": "task", "output_ref": "some-feature" }` -- should succeed (was previously 400)
3. Verify both fields persisted in manifest

### T4: Existing root-cause behavior unchanged
1. Create root-cause investigation
2. Set output_type and output_ref via CLI -- should work as before
3. Verify singular output_ref field set correctly

### T5: Build and test
1. `SDLC_NO_NPM=1 cargo test --all` passes
2. `cargo clippy --all -- -D warnings` passes
