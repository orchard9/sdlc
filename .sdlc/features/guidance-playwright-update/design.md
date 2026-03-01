# Design: Playwright UAT Documentation Updates

## Approach

Documentation-only change. Two files are updated in-place:

### `.sdlc/guidance.md` changes

1. **§5 "Meaningful, Reliable, Fast Tests"** — append a note that the third tier of the project's test strategy is Playwright e2e (deterministic regression layer) as described in `docs/uat-enterprise-strategy.md`.

2. **New §11 "UAT with Playwright"** — added after §10 (Frontend API Calls), covering:
   - Three-tier architecture summary (one paragraph)
   - Spec location: `frontend/e2e/milestones/<slug>.spec.ts`
   - How to run: `cd frontend && npx playwright test`
   - Milestone-specific run: `npx playwright test e2e/milestones/<slug>.spec.ts`
   - Agent UAT modes (Mode A: spec exists, Mode B: generate spec)
   - Reference to `docs/uat-enterprise-strategy.md` for full details

### `CLAUDE.md` changes

1. **Key Files table** — add row: `docs/uat-enterprise-strategy.md` | Three-tier UAT architecture strategy — Playwright regression layer, Playwright MCP agent integration, CI setup

The existing `## Playwright MCP for Milestone UAT` section in `CLAUDE.md` already documents the Playwright MCP tools and modes — no changes needed there.

## No structural changes

No new files. No YAML. No Rust or TypeScript changes. Both edits are additive (append/insert rows).
