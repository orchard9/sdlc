## Brief

During autonomous sdlc runs, agents frequently identify out-of-scope concerns — architectural issues, cross-cutting debt, systemic problems noticed incidentally while fixing something else. These items have no natural home:

- `sdlc task add <slug>` requires a feature slug — these items don't belong to any specific feature
- `sdlc comment --flag fyi` is feature-local and invisible to the next agent session  
- `sdlc comment --flag blocker` gates the pipeline (too heavy for observations)
- `sdlc escalate` requires human action and stops the run (also too heavy)
- Advisory findings (.sdlc/advisory.yaml) are for macro health analysis, not operational session notes

Example from a real session:
> Split-brain registry with 2 replicas — sessions are stored in-memory per pod. When the daemon connects to pod A but a public request hits pod B, pod B 502s. Needs either sticky sessions (Traefik affinity) or a shared registry backend. For now the cluster is scaled to 1 replica which avoids the problem.

This is a real concern, not a human blocker, not feature-scoped, not urgent but should not be lost.

## Goal

Design and implement `sdlc backlog` — a project-level parking lot for out-of-scope concerns with a clear path to promotion (backlog item → feature when it's time to act on it).

## Scope

- New CLI: `sdlc backlog add/list/promote/park`
- New storage: `.sdlc/backlog.yaml`
- Guidance update: when agents write backlog items and when they promote them
- sdlc-run / sdlc-next command updates: mandate writing backlog items at end of run
- Dashboard surface: backlog items visible in the web UI
- All code touchpoints: sdlc-core types, sdlc-cli command, sdlc-server routes, frontend