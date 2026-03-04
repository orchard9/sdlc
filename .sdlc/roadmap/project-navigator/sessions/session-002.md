---
session: 2
timestamp: 2026-03-04T20:00:00Z
orientation:
  current: "Architecture fully locked — heartbeat protocol, credential pool, hub topology for both local and cluster. Written into ARCHITECTURE.md."
  next: "Commit to milestone. This is ready to build."
  commit: "Architecture is documented and aligned. No open questions remain."
---

# Session 2 — Heartbeat model + credential pool alignment

## What changed from session 1

Session 1 designed a config-driven listing with explicit register/deregister and a
Woodpecker git-commit approach for the cluster. Jordan pushed back on the
register/deregister lifecycle and proposed a check-in / heartbeat model instead.

## Heartbeat protocol (replaces config-driven approach)

⚑ Decided: `POST /api/hub/heartbeat` every 30s — first beat = registration, silence = deregistration.
⚑ Decided: Hub sweep every 15s — >90s marks offline, >5min removes entry.
⚑ Decided: No explicit register/deregister calls anywhere. No SIGTERM handling needed.
⚑ Decided: Heartbeat payload carries all card metadata (milestone, feature count, agent running).
⚑ Decided: Hub persists state to disk for restart recovery (cache, not source of truth).

**Cluster simplification:** With heartbeats, Woodpecker pipelines need zero changes for
hub registration. Pod up = beats start = appears. Pod deleted = beats stop = disappears.
No hub config repo, no commit step.

## Credential pool included

Jordan pasted a full architectural plan for the PostgreSQL-backed Claude credential pool.
Integrated into the architecture decisions:

- `claude_credentials` table in shared cluster Postgres (ns: databases, postgres-0)
- `SELECT FOR UPDATE SKIP LOCKED ORDER BY last_used_at ASC` for round-robin checkout
- `CLAUDE_CODE_OAUTH_TOKEN` injected via `QueryOptions` env
- Graceful degradation: no `DATABASE_URL` → skip pool init, run without token
- Helm ExternalSecret pulls `k3sf-postgres-sdlc` from GCP Secret Manager

## Full cluster topology (final)

```
k3s cluster
├── ns: sdlc-<slug> × N        — project pods (sdlc-server + git-sync)
│     each beats to hub ClusterIP every 30s
├── ns: sdlc-hub               — hub pod (sdlc-server --hub + git-sync)
│     ClusterIP, ingress: sdlc.threesix.ai
└── ns: databases              — postgres-0, claude_credentials table
```

## ARCHITECTURE.md updated

Added "Fleet Deployment" section covering:
- Pod model (sdlc-server + git-sync sidecar, emptyDir, no PVCs)
- Hub mode with heartbeat protocol and ASCII cluster diagram
- Credential pool (checkout flow, graceful degradation, Helm secrets)
- Auth (Google OAuth, agent token bypass)
- Updated "What to Read First" to include fleet deploy ponder

## Product Summary

### What we explored
Finalized the registration model for the project navigator hub (heartbeat instead of
explicit register/deregister) and integrated the credential pool architecture so both
features are documented together as a coherent fleet story.

### Key shifts
Moved from config-driven + Woodpecker git-commits to a pure heartbeat model — simpler,
self-correcting, no pipeline changes needed. The credential pool was already designed;
it's now written into the canonical architecture doc alongside the hub.

### Implications
The fleet architecture is now fully documented in ARCHITECTURE.md and ready to build.
Three things ship together: hub mode, heartbeat protocol, credential pool. They're
independent features but part of the same milestone story.

### Still open
Nothing. Architecture is locked. Ready to commit to a milestone.
