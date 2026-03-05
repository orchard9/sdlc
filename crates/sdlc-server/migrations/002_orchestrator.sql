-- Orchestrator actions: scheduled and webhook-triggered tool invocations.
-- trigger and status are stored as JSONB because they are tagged enums
-- (ActionTrigger, ActionStatus) whose JSON representation is the canonical form.
-- recurrence_secs is the Duration serialized as whole seconds (None → NULL).
CREATE TABLE IF NOT EXISTS orchestrator_actions (
    id              UUID PRIMARY KEY,
    label           TEXT NOT NULL,
    tool_name       TEXT NOT NULL,
    tool_input      JSONB NOT NULL,
    trigger         JSONB NOT NULL,
    status          JSONB NOT NULL,
    recurrence_secs BIGINT,
    created_at      TIMESTAMPTZ NOT NULL,
    updated_at      TIMESTAMPTZ NOT NULL
);

-- Raw webhook payloads received via POST /api/orchestrator/webhooks/*path.
-- raw_body is stored as BYTEA because payloads may be binary (e.g. protobuf).
CREATE TABLE IF NOT EXISTS orchestrator_webhooks (
    id           UUID PRIMARY KEY,
    route_path   TEXT NOT NULL,
    raw_body     BYTEA NOT NULL,
    content_type TEXT,
    received_at  TIMESTAMPTZ NOT NULL
);

-- Persistent mappings from a URL path to a tool invocation.
-- path has a UNIQUE constraint because each path may map to at most one route.
-- template stores the raw input_template string (not JSONB — it may contain
-- the {{payload}} placeholder before rendering).
CREATE TABLE IF NOT EXISTS orchestrator_webhook_routes (
    id         UUID PRIMARY KEY,
    path       TEXT UNIQUE NOT NULL,
    tool_name  TEXT NOT NULL,
    template   TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL
);

-- Immutable audit records for every webhook arrival and dispatch outcome.
-- Ring buffer cap of 500 is enforced at the application layer, not in the DB.
-- outcome is stored as JSONB (WebhookEventOutcome tagged enum).
-- seq is a BIGSERIAL so insertions auto-assign a monotonically increasing value.
CREATE TABLE IF NOT EXISTS orchestrator_webhook_events (
    id           UUID PRIMARY KEY,
    seq          BIGSERIAL NOT NULL,
    route_path   TEXT NOT NULL,
    content_type TEXT,
    body_bytes   BIGINT NOT NULL,
    outcome      JSONB NOT NULL,
    received_at  TIMESTAMPTZ NOT NULL
);

CREATE INDEX IF NOT EXISTS orchestrator_webhook_events_seq_idx ON orchestrator_webhook_events (seq);
