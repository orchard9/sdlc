CREATE TABLE IF NOT EXISTS otp_invites (
    id          TEXT PRIMARY KEY,
    email       TEXT NOT NULL,
    otp_hash    BYTEA NOT NULL,
    created_by  TEXT NOT NULL,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at  TIMESTAMPTZ NOT NULL,
    status      TEXT NOT NULL DEFAULT 'pending',
    used_at     TIMESTAMPTZ
);

CREATE INDEX IF NOT EXISTS otp_invites_email_status_idx
    ON otp_invites (email, status) WHERE status = 'pending';

CREATE TABLE IF NOT EXISTS otp_attempts (
    email        TEXT NOT NULL,
    attempted_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS otp_attempts_email_time_idx
    ON otp_attempts (email, attempted_at);
