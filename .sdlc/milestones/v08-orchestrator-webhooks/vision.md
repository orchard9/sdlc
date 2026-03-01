# v08 Vision: Webhook Ingestion and Routing

External systems can trigger tool runs by posting to a registered webhook endpoint.
The payload is stored raw and processed on the next tick — no polling, no long-lived
connections, no coordination required.

## What a user can do when this ships

- Register a webhook route: `POST /api/orchestrator/webhooks/routes { path: "/webhooks/deploy", tool_name: "deploy-tool", input_template: "{{payload}}" }`
- Trigger it from any external system: `curl -X POST https://sdlc-server/webhooks/deploy -d '{"ref":"main"}'`
- The tool fires on the next tick with the payload as input
- View the triggered action in `sdlc orchestrate list`

## Why it matters

Services need to react to external events — a GitHub push, a CI result, a Slack
message. Webhooks make the orchestrator event-driven without adding queue
infrastructure. The raw storage model means the orchestrator never loses a signal,
and the tick-rate model means processing is always deliberate and bounded.
