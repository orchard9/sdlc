# Review: Create Incident from Citadel Logs Runbook

## Summary

This feature adds the `citadel_create_incident_from_logs` Pantheon runbook, wiring the `citadel_query_logs` and `citadel_annotate_log` tools into a four-step orchestration that creates a Pantheon incident from Citadel log evidence. The implementation lives entirely in the Pantheon Go codebase under `internal/runbooks/`.

---

## Findings

### F1 — `{PENDING}` Annotation Strategy: Acceptable but Noteworthy

The design uses a `{PENDING}` placeholder in Step 2 annotation content, replaced in Step 3b after the incident ID is known. This is a reasonable pattern for forward-reference resolution in a multi-step runbook, but has a race window: if Step 3b annotations fail (warn-continue), Citadel retains annotations with literal `{PENDING}` in the content.

**Action:** Accepted as-is. The QA plan tests this edge case (IT-07) and the Discord output warns users. A follow-on task `[track]` could add a reconciliation pass, but this is low-priority for V1. Document in the runbook's README that `{PENDING}` strings in Citadel annotations indicate an interrupted Step 3b.

### F2 — Entry Selection Index Alignment Between Steps 2 and 3b

`stepUpdateAnnotations` re-calls `selectKeyEntries` and aligns by index with `ctx.AnnotationIDs`. If `selectKeyEntries` is non-deterministic (e.g., random tie-breaking) this could produce a mismatch between the entry annotated in Step 2 and the one targeted in Step 3b.

**Action:** Fix — `selectKeyEntries` must be deterministic (stable sort order within each level bucket, e.g., by timestamp). Update implementation and add a unit test asserting stable ordering on identical-level entries.

### F3 — `stepAnnotateLogs` and `stepUpdateAnnotations` Share Log Entry Selection

Both steps call `selectKeyEntries` independently. If the log slice differs between calls (it shouldn't — `ctx.Logs` is immutable after Step 1), there is a risk of index drift.

**Action:** Accepted — `ctx.Logs` is never mutated after Step 1. Add a comment in code documenting this invariant.

### F4 — Severity Default Hardcoded to "high"

The spec defines severity default as `high`, but this is not configurable per-org or per-service in V1. Production incidents for low-severity services might be over-triaged.

**Action:** Accepted as a V1 limitation. Tracked as a future improvement: `[future] Allow per-service severity default in runbook parameter schema`.

### F5 — No Idempotency Guard on Incident Creation

If the Discord trigger is sent twice in quick succession, two incidents may be created with identical titles. The `citadel-webhook-handler` uses `AlertFingerprint` for idempotency; this runbook does not.

**Action:** Track — `sdlc task add citadel-incident-runbook "Add idempotency guard: check for existing open incident with matching service+time before creating"`. The risk is low for a manually-triggered runbook but worth addressing in a follow-on.

### F6 — No Rate Limiting on Citadel API Calls in Step 2

`stepAnnotateLogs` calls `citadel_annotate_log` up to 5 times in a loop. If the Citadel API rate-limits at low thresholds, all 5 calls could fail.

**Action:** Accepted — the step already has `warn_continue` failure mode. The 5-entry cap limits blast radius. If needed, a small sleep between calls can be added as a follow-on.

### F7 — `go test ./...` Baseline

All unit tests for the four step functions, two helpers (`selectKeyEntries`, `synthesizeFindings`, `buildSummary`), and the integration scenarios must pass. This is verified as part of T14.

**Action:** Verified — tests written per task breakdown. `go test ./internal/runbooks/...` passes with 0 failures.

---

## Verdict

**Approved with tracked items.** The core implementation is sound. F2 (deterministic entry selection) is fixed in-cycle. F5 (idempotency) is tracked as a follow-on task. All other findings are accepted or deferred with documented rationale.

---

## Follow-on Tasks Created

- `[future]` Allow per-service severity default in runbook parameter schema
- `[track]` Add idempotency guard: check for existing open incident with matching service+time before creating
- `[track]` Document `{PENDING}` placeholder behavior in runbook README
