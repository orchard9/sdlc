# Pantheon API Surface — Integration-Relevant Endpoints

## Identity
Developer hub platform. Go service. Discord-first but has web UI + CLI.
Primary use: AI agents + developers collaborating on operational work.
Hosted on k8s (GCP). Auth: Discord OAuth + API keys + device auth (for CLI).

## Authentication
- **Session cookie**: Web users after Discord OAuth
- **Service key**: Discord bot (static key → full org access)
- **API key format**: Org-scoped token in header — each key cannot access other orgs
- **Device auth**: CLI flow (request → user verifies code → poll → access token)

## Task Management
```
GET/POST /api/v1/organizations/{orgSlug}/projects/{projectSlug}/tasks/
GET/PATCH/DELETE /api/v1/organizations/{orgSlug}/projects/{projectSlug}/tasks/{taskId}
```
Task statuses: backlog → todo → in_progress → review → blocked → done | wont_do | cancelled
Task priorities: critical | high | medium | low

## Incident Management
```
POST /api/v1/organizations/{orgSlug}/incidents/           # Create incident
GET  /api/v1/organizations/{orgSlug}/incidents/           # List incidents
POST /api/v1/organizations/{orgSlug}/incidents/{id}/resolve
     severity: critical | high | medium | low
     status: investigating → identified → mitigating → resolved
```

## Agent Tool Registry (App Platform)
Pantheon has a built-in tool definition + credential system:
```
ToolDefinition    # JSON schema definition for an external API call
ToolCredential    # Encrypted API credentials scoped to a tool
Tool execution    # Agent calls tool, Pantheon handles auth + HTTP
Approval gates    # Human approval required for high-risk tool calls
```
This is exactly the right layer for Citadel integration — register Citadel as an App.

## Agent Capabilities (Tool Categories)
Current tools: tasks, incidents, runbooks, K8s cluster ops, deployments.
Missing: observability / log inspection — THIS IS THE GAP Citadel fills.

## Runbooks
```
GET  /api/v1/organizations/{orgSlug}/runbooks/
POST /api/v1/organizations/{orgSlug}/runbooks/{id}/execute
     Steps: command | manual | verify | approval
```
Runbooks with Citadel steps would be powerful: query logs → analyze → create task.

## Memory / Recall
```
GET /api/v1/memory/    # Query conversation history (Synap vector DB)
```
Agent memory uses 20k token budget: 10k recent + 6k summaries + 4k recalled.

## CLI (Go, Cobra)
`pantheon task list|create|update`
`pantheon incident create|resolve`
Device auth flow for CLI → access token