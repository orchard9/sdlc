# v07 Vision: Tick-Rate Orchestrator Core

A developer runs `sdlc orchestrate` and walks away. Tools run on schedule, results
are stored, and the system recovers cleanly from restarts — all without human
intervention.

## What a user can do when this ships

- Schedule any sdlc tool to fire at a specific time: `sdlc orchestrate add my-svc --tool sdlc-next --input '{"slug":"my-svc"}' --at now+1h`
- Start the orchestrator daemon: `sdlc orchestrate --tick-rate 60`
- Inspect action history: `sdlc orchestrate list`
- Restart the daemon at any time without losing or re-firing actions

## Why it matters

This is the foundation of autonomous service operation. Every service Jordan builds
over the next 3 months can be driven by a scheduled action — `sdlc next --for <slug>`
runs on a tick, dispatches an agent, advances the feature. One daemon, 100 services,
zero human coordination.

The tool-as-executor model means every tool Jordan builds improves both the UI
workflow and the scheduled workflow simultaneously. Tools and orchestrator co-evolve.
