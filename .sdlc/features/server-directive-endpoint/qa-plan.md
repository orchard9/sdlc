# QA Plan: server-directive-endpoint

## Test Coverage

### Unit / Integration Tests (Rust)

1. **Happy path** — `GET /api/features/:slug/directive` for a feature in draft phase:
   - Returns HTTP 200
   - Response JSON contains all Classification fields: `feature`, `title`, `current_phase`, `action`, `message`, `next_command`, `is_heavy`, `timeout_minutes`
   - `description` field present when the feature has a description set

2. **Not found** — `GET /api/features/nonexistent/directive`:
   - Returns HTTP 404 or error response (via `AppError`)

3. **Field completeness vs `/next`** — response from `/directive` must include at minimum all fields returned by `/next`, plus `description` when non-null.

## Manual Smoke Test

```bash
# Start server
cargo run -p sdlc-server

# Hit the endpoint
curl -s http://localhost:8484/api/features/server-directive-endpoint/directive | jq .
```

Expected: JSON matching `sdlc next --for server-directive-endpoint --json` output.
