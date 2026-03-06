# Code Review: Webhook Payload Inspector UI

## Summary

This feature extends the existing webhook infrastructure UI with payload inspection and replay capabilities. All 5 tasks were completed:

1. **T1** -- Extended `OrchestratorWebhookRoute` with `store_only` and `secret_token` fields; added `WebhookPayloadItem` type.
2. **T2** -- Added `queryWebhookPayloads` and `replayWebhookPayload` API client methods.
3. **T3** -- Updated `WebhookRoutesSection` with store-only chip, lock icon, and Inspect button.
4. **T4** -- Built `WebhookPayloadInspector` component with two-pane layout, time filtering, JSON viewer, copy, and replay.
5. **T5** -- Added `POST /api/webhooks/{route}/replay/{id}` endpoint in Rust backend.

## Files Changed

| File | Change |
|------|--------|
| `frontend/src/lib/types.ts` | Added `store_only`, `secret_token` to `OrchestratorWebhookRoute`; added `WebhookPayloadItem` interface |
| `frontend/src/api/client.ts` | Added `queryWebhookPayloads` and `replayWebhookPayload` methods |
| `frontend/src/pages/ActionsPage.tsx` | Extended `WebhookRoutesSection` with indicators + inspector toggle; added import for `WebhookPayloadInspector` |
| `frontend/src/components/webhooks/WebhookPayloadInspector.tsx` | New component (two-pane payload browser) |
| `crates/sdlc-server/src/routes/webhooks.rs` | Added `replay_webhook` handler + `uuid` import |
| `crates/sdlc-server/src/lib.rs` | Registered `/api/webhooks/{route}/replay/{id}` route |

## Review Findings

### Finding 1: Replay endpoint uses wide time-range scan (Accepted)
The replay endpoint queries 30 days of payloads to find a single payload by UUID because the `OrchestratorBackend` trait lacks a `get_webhook_by_id` method. This is acceptable for the current scale -- store-only routes typically have low-volume payloads. If volume grows, adding a `get_by_id` method to the trait would be a targeted optimization.

### Finding 2: No pagination in payload inspector (Accepted)
The inspector fetches up to 100 payloads at a time. For typical webhook volumes (tens per day), this is sufficient. If high-volume routes emerge, pagination with cursor support would be needed. This is out of scope per the spec.

### Finding 3: Replay does not track run lifecycle (Accepted)
The replay generates a UUID `run_id` but this is not a RunRecord tracked in the telemetry system. It's a synchronous tool invocation. For full observability, a future enhancement could integrate with `spawn_agent_run` or create a proper RunRecord. This is consistent with how the orchestrator tick loop dispatches webhooks today.

### Finding 4: Error handling is consistent
Both the frontend and backend handle errors gracefully -- the inspector shows inline error messages, the replay result strip shows success/failure, and the backend returns appropriate HTTP status codes.

## Build Verification

- `SDLC_NO_NPM=1 cargo check --all` -- passes
- `cargo clippy --all -- -D warnings` -- passes (no warnings)
- `npx tsc --noEmit` -- passes (no type errors)

## Verdict

**Approved.** All changes are well-scoped, follow existing patterns, and compile cleanly. The three accepted findings are documented trade-offs appropriate for the current scale.
