CREATE TABLE IF NOT EXISTS run_records (
    id           TEXT PRIMARY KEY,
    key          TEXT NOT NULL,
    run_type     TEXT NOT NULL,
    target       TEXT NOT NULL,
    label        TEXT NOT NULL,
    status       TEXT NOT NULL,
    started_at   TEXT NOT NULL,
    completed_at TEXT,
    cost_usd     DOUBLE PRECISION,
    turns        BIGINT,
    error        TEXT,
    prompt       TEXT,
    session_id   TEXT,
    stop_reason  TEXT
);

CREATE TABLE IF NOT EXISTS run_events (
    run_id TEXT NOT NULL REFERENCES run_records(id) ON DELETE CASCADE,
    events JSONB NOT NULL,
    PRIMARY KEY (run_id)
);
