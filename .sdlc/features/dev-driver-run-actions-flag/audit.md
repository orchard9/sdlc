# Security Audit: dev-driver-run-actions-flag

## Scope

This audit covers the security implications of inverting the orchestrator-start default in `sdlc ui` — from opt-out (`--no-orchestrate`) to opt-in (`--run-actions`).

---

## Security Surface Analysis

### What changed

The orchestrator daemon starts **only** when `--run-actions` is explicitly passed. Previously it started by default unless `--no-orchestrate` was passed.

### What the orchestrator does

The orchestrator daemon (`orchestrate::run_daemon`) runs scheduled actions. These actions can include:
- Spawning Claude agents (via `dev-driver` tool)
- Executing arbitrary `sdlc next` directives on features

### Security implications of the inversion

| Concern | Before (default-on) | After (default-off) | Assessment |
|---|---|---|---|
| Unintended action execution | High: orchestrator runs on every `sdlc ui` startup | None: orchestrator only runs when `--run-actions` explicitly passed | **Improved** |
| Privilege escalation | No change — orchestrator runs as the same user | No change | Neutral |
| Silent agent spawning | High: agent runs could happen without developer awareness | None: requires explicit flag | **Improved** |
| Action audit trail | Unchanged — runs.json persisted in .sdlc/ | Unchanged | Neutral |

### Attack surface

No new attack surface is introduced. The feature is a flag rename and semantic inversion. It does not:
- Introduce new network endpoints
- Change how the orchestrator processes actions
- Add new permissions or capabilities
- Modify authentication or authorization logic

### Trust model alignment

The change aligns the system with the principle of least surprise: an AI-agent execution system should not execute agents unless the operator has affirmatively opted in. This reduces the blast radius of accidental `sdlc ui` invocations in CI pipelines, shared machines, or automated scripts.

---

## Findings

| # | Severity | Finding | Disposition |
|---|---|---|---|
| S1 | POSITIVE | Default-off is strictly safer than default-on for autonomous action execution | No action needed |
| S2 | INFO | The `--run-actions` flag is not authenticated — anyone who can run `sdlc ui --run-actions` on the machine can start the orchestrator | Acceptable: `sdlc` is a local developer tool; process-level access control is the machine's responsibility |
| S3 | INFO | Backlog item B7 (config-file escape hatch) would allow project-wide `run_actions: false` enforcement — not yet implemented | Tracked as future work |

No security blockers identified.

---

## Audit Decision

**APPROVED.** This change strictly improves the security posture by making autonomous action execution opt-in. No new attack surface. No regressions in existing security controls.
