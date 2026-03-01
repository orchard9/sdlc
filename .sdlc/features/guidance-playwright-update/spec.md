# Spec: Update guidance.md and CLAUDE.md with Playwright UAT Pattern

## Problem

The project has adopted a three-tier UAT architecture using Playwright for deterministic regression testing, with `@microsoft/playwright-mcp` registered in `.mcp.json` and specs living in `frontend/e2e/milestones/`. Neither `.sdlc/guidance.md` nor `CLAUDE.md` currently document this pattern. Agents reading these files have no knowledge of the Playwright UAT workflow, spec locations, or how to run tests.

## Goal

Update `.sdlc/guidance.md` and `CLAUDE.md` to document:
1. The three-tier UAT architecture (guidance.md §5 and a new §11)
2. Where Playwright specs live (`frontend/e2e/milestones/<slug>.spec.ts`)
3. How to run Playwright tests (`cd frontend && npx playwright test`)
4. That `.mcp.json` registers `@microsoft/playwright-mcp` for agent UAT runs
5. Reference `docs/uat-enterprise-strategy.md` as the canonical strategy doc
6. Add `docs/uat-enterprise-strategy.md` to the Key Files table in CLAUDE.md

## Out of Scope

- No changes to Rust code or TypeScript
- No new features — documentation only
- No changes to the state machine, rules, or CLI

## Acceptance Criteria

- `.sdlc/guidance.md` §5 references the three-tier UAT architecture and Playwright as the regression layer
- `.sdlc/guidance.md` has a new §11 UAT section with: spec location, run commands, Mode A/B, and link to strategy doc
- `CLAUDE.md` Key Files table includes `docs/uat-enterprise-strategy.md`
- Both files accurately reflect what's implemented (`.mcp.json`, `frontend/e2e/milestones/`, `CLAUDE.md` Playwright MCP section already present)
