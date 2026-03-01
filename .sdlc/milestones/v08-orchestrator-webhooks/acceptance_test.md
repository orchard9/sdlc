# Acceptance Test: v08-orchestrator-webhooks

## Setup

```bash
# Start sdlc-server and orchestrator
sdlc ui &
sdlc orchestrate --tick-rate 15
```

## Scenario 1: Register a webhook route

```bash
curl -X POST http://localhost:<port>/api/orchestrator/webhooks/routes \
  -H "Content-Type: application/json" \
  -d '{"path":"/webhooks/test","tool_name":"quality-check","input_template":"{}"}'
```

**Expected:** 201 response. Route appears in `GET /api/orchestrator/webhooks/routes`.

## Scenario 2: Fire a webhook and verify processing

```bash
curl -X POST http://localhost:<port>/webhooks/test \
  -H "Content-Type: application/json" \
  -d '{"event":"push","ref":"main"}'
```

**Expected:**
1. 202 response (payload stored)
2. Within 15 seconds, `sdlc orchestrate list` shows a `Completed` action triggered
   by the webhook
3. The action's tool_name is `quality-check`

## Scenario 3: Unregistered webhook route

```bash
curl -X POST http://localhost:<port>/webhooks/unknown -d '{}'
```

**Expected:** 202 response (stored). No action fires (no route registered).
No crash or error in daemon logs.

## Scenario 4: Payload mapping

Register route with `input_template: '{"source":"webhook","body":"{{payload}}"}'`.
Post any JSON body. Verify the tool receives `{"source":"webhook","body":"<raw>"}`.
