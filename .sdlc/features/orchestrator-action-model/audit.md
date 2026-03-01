# Security Audit: orchestrator-action-model

## Surface

Pure data layer — no HTTP endpoints, no process spawning, no user input
processing in this feature. The security surface is limited to:

1. **DB file path** — caller-controlled via `ActionDb::open(path)`
2. **Stored `tool_input`** — arbitrary `serde_json::Value` stored as-is
3. **`raw_payload`** on `ActionTrigger::Webhook` — arbitrary bytes stored as-is

## Findings

**No critical issues.** This is a storage-only module.

### Path traversal (low risk, by design)

`ActionDb::open(path)` accepts any `&Path`. The caller (`sdlc orchestrate`)
controls this path. In practice it defaults to `.sdlc/orchestrator.db`. No
path sanitization is needed here — the CLI is the trust boundary, not the DB.

### Tool input storage (informational)

`tool_input` is stored verbatim and will later be passed to `run_tool()` as
stdin JSON. The security boundary for `tool_input` validation belongs in the
CLI (`sdlc orchestrate add`) and the tool execution layer — not in the DB
module, which is correctly dumb about what it stores.

### Webhook raw payload (informational)

`raw_payload: Vec<u8>` stores arbitrary bytes. No deserialization happens at
storage time (no injection surface here). The injection risk exists at process
time when the payload is rendered into a tool input template — that is a Phase 2
concern, not in scope for this feature.

### redb ACID guarantees

redb provides ACID transactions. The `set_status` pattern (remove + reinsert
in a single write transaction) is atomic. A crash between remove and insert
is not possible — redb rolls back incomplete transactions on open.

## No issues requiring action in this feature.
