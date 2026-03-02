# Onboarding + Dashboard Improvement Plan

## Purpose
This document supports `onboarding-dashboard-mockup.html` and explains:
1. what the current product already supports,
2. what is missing for the proposed experience,
3. how to close those gaps in implementation order.

## What We Already Support

| Area | Current Support | Evidence |
|---|---|---|
| Lifecycle engine | Deterministic directive loop and phase progression | `sdlc next --for <slug> --json`, `sdlc artifact draft/approve`, architecture docs |
| Core onboarding context | Setup flow for project description, vision, architecture, team recruitment | `frontend/src/pages/SetupPage.tsx` |
| Operational dashboard base | Project overview, feature stats, blockers, escalations, active work visibility | `frontend/src/pages/Dashboard.tsx` |
| Milestone and feature controls | Feature/milestone creation and inspection are already first-class | CLI (`sdlc feature *`, `sdlc milestone *`) + API client |
| Live updates | SSE-based refresh and run lifecycle events | `/api/events`, `useSSE`, architecture docs |
| Query surfaces | Blocked/ready/needs-approval and search commands available | `sdlc query blocked|ready|needs-approval|search` |

## What Is Missing vs Proposed UX

### 1) Unified "first 15 minutes" onboarding run
- Missing: a single linear flow from detection -> plan scaffold -> first successful directive loop.
- Current state: setup is strong for project context, but not tightly connected to first real lifecycle execution.

### 2) Focused execution queue in one place
- Missing: a canonical ranked "Next Action Queue" combining phase urgency, blockers, and review load.
- Current state: signals exist, but are scattered across feature cards, milestone pages, and query commands.

### 3) Review inbox ergonomics
- Missing: dedicated approval inbox with one-click open/approve/fix loops from dashboard.
- Current state: approvals are visible indirectly through feature actions and query surfaces.

### 4) Onboarding auto-detection summary
- Missing: explicit diagnostic card for prerequisites (missing docs, missing agents, missing secrets/tool setup).
- Current state: pieces are detectable but not exposed as one onboarding readiness report.

### 5) Contract drift guardrail
- Missing: automated checks that product prompts, docs, and CLI commands remain consistent.
- Current state: drift can occur between prompts and supported command flags/subcommands.

## Gap Closure Plan

## Phase 1: Onboarding Funnel (high impact)
Goal: reduce time-to-first-successful-transition.

1. Add `OnboardingFlowPage` (`/onboarding`) that composes:
   - existing setup sections,
   - readiness checks,
   - first-run directive executor panel.
2. Add backend readiness endpoint:
   - `GET /api/onboarding/readiness`
   - returns booleans + recommended next command.
3. Add "first feature scaffold" step:
   - create milestone + 1-3 starter features from one form,
   - immediately run `sdlc next --for <slug> --json` and present actions.

## Phase 2: Dashboard Control Tower
Goal: improve operational clarity and reduce navigation hops.

1. Add aggregated queue endpoint:
   - `GET /api/dashboard/queue`
   - merges directives + blockers + approvals + escalation state.
2. Add dashboard sections from mockup:
   - Focus Now,
   - Needs Approval,
   - Blockers,
   - Next Action Queue,
   - Active Runs,
   - Activity Feed.
3. Add quick actions with direct command affordances:
   - open directive,
   - run focused feature,
   - open review queue.

## Phase 3: Reliability + Consistency Hardening
Goal: keep autonomous flow trustworthy.

1. Add prompt/CLI contract tests for run prompt templates and docs snippets.
2. Add UI + server safety polish:
   - UTF-8-safe text truncation,
   - consistent slug validation path.
3. Add telemetry counters:
   - onboarding drop-off point,
   - time to first approved artifact,
   - queue age for approval items.

## Success Metrics

- **Time to first approved artifact**: target under 15 minutes from onboarding start.
- **Approval throughput**: median age of pending approvals reduced by 30%.
- **Navigation compression**: fewer page hops to complete one directive cycle.
- **Autonomous run reliability**: lower failure rate from invalid command/flag usage.

## Delivery Notes

- Prototype file: `html-prototypes/onboarding-dashboard-mockup.html`
- This plan is intentionally additive to existing UI architecture (`SetupPage`, `Dashboard`, SSE hooks), not a rewrite.
