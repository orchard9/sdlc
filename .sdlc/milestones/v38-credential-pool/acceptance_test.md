# Acceptance Test: v38-credential-pool

## Setup

Requires a running Postgres instance with the `claude_credentials` table (auto-created
on first pod start). Set `DATABASE_URL` env var pointing to it.

## Scenarios

### 1. Pool initializes

- [ ] Pod starts with `DATABASE_URL` set
- [ ] Logs show "credential pool ready" (or equivalent)
- [ ] `claude_credentials` table is created if it didn't exist

### 2. Graceful degradation — no rows

- [ ] `claude_credentials` table is empty
- [ ] Trigger an agent run from the UI
- [ ] Run proceeds normally (no crash, no hang)
- [ ] Logs show warn: "no active Claude credentials"
- [ ] `CLAUDE_CODE_OAUTH_TOKEN` is NOT set in the subprocess (ambient auth fallback)

### 3. Token checkout and injection

- [ ] Insert one row: `INSERT INTO claude_credentials (account_name, token) VALUES ('test@example.com', 'tok_abc')`
- [ ] Trigger an agent run
- [ ] Logs show token was checked out (account_name logged)
- [ ] `last_used_at` on the row is updated after the run
- [ ] `use_count` on the row increments

### 4. Round-robin with two tokens

- [ ] Insert two rows with different tokens
- [ ] Run two sequential agent runs
- [ ] First run uses the token with older `last_used_at`
- [ ] Second run uses the other token
- [ ] `last_used_at` alternates — confirmed by querying the table

### 5. Concurrent checkout — no blocking

- [ ] Insert two rows
- [ ] Trigger two agent runs concurrently (e.g., two features, two ponder sessions)
- [ ] Each run gets a different token (SELECT FOR UPDATE SKIP LOCKED)
- [ ] Neither run waits for the other

### 6. Graceful degradation — DB unreachable

- [ ] Start server with `DATABASE_URL` pointing to an unreachable host
- [ ] Server starts (does not crash or refuse to boot)
- [ ] Warn logged at startup: credential pool unavailable
- [ ] Agent runs proceed without token injection (ambient auth)

### 7. Helm — DATABASE_URL injected in cluster

- [ ] Deploy a project pod with the updated Helm chart
- [ ] `kubectl exec` into the pod — confirm `DATABASE_URL` env var is present
- [ ] ExternalSecret `postgres-sdlc-credentials` exists in the namespace
- [ ] GCP Secret `k3sf-postgres-sdlc` is the source
