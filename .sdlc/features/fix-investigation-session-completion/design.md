# Design: Fix Investigation Session Completion

## Changes

### 1. CLI: Add `--output-type` flag (investigate.rs)

Add `output_type: Option<String>` to `InvestigateSubcommand::Update`. In `update()`:
- Validate it's root-cause only
- Set `entry.output_type = Some(ot)`

### 2. CLI: Allow `--output-ref` for evolve (investigate.rs)

Change the `--output-ref` match arm:
```
RootCause => entry.output_ref = Some(oref.to_string()),
Evolve => entry.output_refs.push(oref.to_string()),
Guideline => entry.publish_path = Some(oref.to_string()),
```

Remove the bail for evolve.

### 3. REST API: Allow output_type/output_ref for evolve (investigations.rs)

Change the `output_type` guard from `!= RootCause` to allow both RootCause and Evolve.
For Evolve, set `output_type` on the entry directly (the struct already has the field as `Option<String>`).

Change the `output_ref` guard similarly:
- RootCause: set `entry.output_ref`
- Evolve: set `entry.output_ref` (for the UI single-output pattern) AND push to `entry.output_refs`

## Data Model

No changes. `InvestigationEntry` already has all needed fields:
- `output_type: Option<String>` -- used by both root-cause and evolve
- `output_ref: Option<String>` -- single output ref (root-cause primary, evolve convenience)
- `output_refs: Vec<String>` -- multi-output (evolve)

## No Frontend Changes

The frontend `EvolveOutputGate.tsx` and `OutputGate.tsx` already send the correct payloads. They fail only because the backend rejects them.
