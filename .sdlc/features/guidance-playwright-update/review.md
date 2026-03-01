# Review: guidance-playwright-update

## Changes Made

### `.sdlc/guidance.md`

**§5 "Meaningful, Reliable, Fast Tests"** — Added a three-tier UAT architecture summary paragraph referencing Tier 1 (agent synthesis), Tier 2 (Playwright regression), and Tier 3 (exploratory/visual). Forward-references §11 and `docs/uat-enterprise-strategy.md`.

**§11 "UAT with Playwright"** (new section) — Documents:
- `@microsoft/playwright-mcp` registered in `.mcp.json`
- Spec location: `frontend/e2e/milestones/<slug>.spec.ts`
- Command table: run all tests, run one milestone, debug, codegen, show-report
- Playwright browser install prerequisite
- Mode A (spec exists) and Mode B (no spec) workflows
- Link to `docs/uat-enterprise-strategy.md`

### `CLAUDE.md`

**Key Files table** — Added row: `docs/uat-enterprise-strategy.md` with description "Three-tier UAT architecture — Playwright regression layer, Playwright MCP agent integration, CI setup".

## Verification

- Read `.sdlc/guidance.md` — §5 update present and accurate, §11 present with all required content
- Read `CLAUDE.md` — Key Files table includes `docs/uat-enterprise-strategy.md`
- No YAML files were directly edited
- No regressions in adjacent sections

## Verdict: Approved

All changes are additive, accurate, and consistent with the implemented system (`.mcp.json`, `frontend/e2e/milestones/`, existing `CLAUDE.md` Playwright MCP section).
