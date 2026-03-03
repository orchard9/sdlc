# Feature Index

Quick-reference index for agent-readable feature traces. Each entry maps a feature name to its lookup doc.

## Features

| Feature | Doc | Entry Points |
|---|---|---|
| App Tunnel Feedback FAB (Reverse HTTP Proxy) | [app-tunnel-feedback.md](app-tunnel-feedback.md) | `routes/app_tunnel.rs:54`, `proxy.rs:115`, `routes/feedback.rs:14` |
| Tools (custom agent tools — create, run, manage) | [tools.md](tools.md) | `routes/tools.rs:20`, `tool_runner.rs:141`, `ToolsPage.tsx:13`, `runs.rs:1844` |
| Actions (orchestrator: scheduled actions, webhook routes, event history) | [actions.md](actions.md) | `routes/orchestrator.rs:192`, `cmd/orchestrate.rs:1`, `ActionsPage.tsx:747`, `orchestrator/db.rs:1` |
