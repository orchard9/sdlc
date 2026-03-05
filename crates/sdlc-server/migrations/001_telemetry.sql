CREATE TABLE IF NOT EXISTS telemetry_events (
    run_id TEXT NOT NULL,
    seq    BIGSERIAL NOT NULL,
    event  JSONB NOT NULL,
    PRIMARY KEY (run_id, seq)
);

CREATE INDEX IF NOT EXISTS telemetry_events_run_id_idx ON telemetry_events (run_id);
