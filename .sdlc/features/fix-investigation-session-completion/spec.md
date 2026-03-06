# Spec: Fix Investigation Session Completion

## Problem

Investigation sessions for root-cause and evolve kinds cannot complete their output phase properly. Three related bugs prevent the full lifecycle from working:

### Bug 1: Evolve output fields rejected by REST API

The `PUT /api/investigations/:slug` handler (investigations.rs lines 248-265) explicitly rejects `output_type` and `output_ref` for non-root-cause investigations. However, the frontend `EvolveOutputGate.tsx` sends exactly these fields when the user clicks "Create Evolution Feature". The REST call fails silently, leaving the evolve investigation unable to record its output.

### Bug 2: Root-cause `output_type` missing from CLI

The CLI `sdlc investigate update` command has no `--output-type` flag. The agent prompt in `start_investigation_chat` tells agents to set output_type but provides no way to do so from the CLI. Only the REST API can set it, but agents use CLI commands.

### Bug 3: Evolve `output_refs` Vec unreachable

The `InvestigationEntry` struct has an `output_refs: Vec<String>` field for evolve investigations, but there is no CLI flag or REST API field to add entries to it. The CLI `--output-ref` flag explicitly rejects evolve investigations with a bail.

## Solution

### Fix 1: Allow `output_type` and `output_ref` for evolve investigations in REST API

Remove the root-cause-only restriction on `output_type` and `output_ref` in the REST `update_investigation` handler. Both root-cause and evolve investigations produce outputs (features/tasks). Guideline investigations use `publish_path` instead, so they can remain excluded if desired or also be allowed.

### Fix 2: Add `--output-type` flag to CLI `sdlc investigate update`

Add an `--output-type` argument (valid values: "task", "guideline") to the `InvestigateSubcommand::Update` variant and handle it in the `update()` function. Restrict to root-cause only (evolve uses a different pattern).

### Fix 3: Allow `--output-ref` for evolve investigations

Remove the bail for evolve in the `--output-ref` handler. For evolve, append to `output_refs` Vec. For root-cause, set `output_ref` (singular). For guideline, set `publish_path` (existing behavior).

## Scope

- `crates/sdlc-cli/src/cmd/investigate.rs` -- add `--output-type` flag, fix `--output-ref` for evolve
- `crates/sdlc-server/src/routes/investigations.rs` -- allow output_type/output_ref for evolve
- `crates/sdlc-core/src/investigation.rs` -- no changes needed (data model already correct)

## Out of Scope

- Frontend changes (the UI components already handle these fields correctly; they just fail at the API layer)
- Phase auto-advancement on completion (separate concern)
- Guideline completion flow (already works via `publish_path`)
