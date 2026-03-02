use crate::cmd::init::registry::CommandDef;

const SDLC_SETUP_QUALITY_GATES_COMMAND: &str = r#"---
description: Set up pre-commit hooks for this project — detect languages, install auto-fix and verification checks
argument-hint: <setup|update|check|fix>
allowed-tools: Bash, Read, Write, Edit, Glob, Grep
---

# sdlc-setup-quality-gates

Detect the project's languages and install pre-commit hooks with auto-fix and verification phases. Quality is enforced at commit time — the hook runs automatically on every `git commit`, ensuring no broken code ever reaches the repo.

> **Before acting:** read `.sdlc/guidance.md` — especially §6 "Using sdlc". Never edit `.sdlc/` YAML directly. <!-- sdlc:guidance -->

## Modes

| Mode | What it does |
|---|---|
| `setup` | Fresh install — detect languages, create `.git/hooks/pre-commit` |
| `update` | Read existing hook, identify gaps against the checklist, add missing checks |
| `check` | Audit existing hook — report what's configured, what's missing, what's slow |
| `fix` | Run all auto-fix tools on the entire codebase right now (not just staged files) |

---

## Tool Matrix

| Check | Go | TypeScript | Rust | Python |
|-------|-----|------------|------|--------|
| Format | gofmt | prettier | rustfmt | black/ruff |
| Imports | goimports | eslint-plugin-import | rustfmt | isort/ruff |
| Lint | golangci-lint | eslint | clippy | ruff |
| Types | compiler | tsc | compiler | mypy/pyright |
| Complexity | gocyclo | eslint complexity | clippy | radon/ruff |

## Threshold Defaults

| Metric | Default | Rationale |
|--------|---------|-----------|
| File length | 500 lines | Fits in head |
| Function length | 100 lines | Single responsibility |
| Cyclomatic complexity | 15-25 | Testable |
| Max pre-commit time | 10s | Won't get disabled |

---

## Steps

### 1. Resolve mode

Get mode from `$ARGUMENTS`. Default to `setup` if none provided.

### 2. Detect languages

```bash
ls go.mod Cargo.toml package.json pyproject.toml 2>/dev/null
```

Build a list of detected languages and their corresponding tools.

### 3. Check existing hooks

```bash
cat .git/hooks/pre-commit 2>/dev/null
```

### 4. Execute based on mode

#### setup (fresh install)

Create `.git/hooks/pre-commit` with two-phase approach:

**Phase 1: Auto-fix** — run formatters on staged files, run linters with `--fix`, re-stage fixed files.

**Phase 2: Verify** — check formatting (should pass after phase 1), run linting (unfixable issues), type check, file length check, complexity check.

```bash
#!/bin/bash
set -e

# Get staged files by type
staged_by_ext() { git diff --cached --name-only --diff-filter=ACM | grep -E "$1" || true; }

STAGED_GO=$(staged_by_ext '\.go$')
STAGED_TS=$(staged_by_ext '\.(ts|tsx)$')
STAGED_RS=$(staged_by_ext '\.rs$')
STAGED_PY=$(staged_by_ext '\.py$')

# Phase 1: Auto-fix
[[ -n "$STAGED_GO" ]] && gofmt -w $STAGED_GO && git add $STAGED_GO
[[ -n "$STAGED_TS" ]] && npx prettier --write $STAGED_TS && npx eslint --fix $STAGED_TS 2>/dev/null; git add $STAGED_TS
[[ -n "$STAGED_RS" ]] && rustfmt $STAGED_RS && git add $STAGED_RS
[[ -n "$STAGED_PY" ]] && ruff format $STAGED_PY && ruff check --fix $STAGED_PY && git add $STAGED_PY

# Phase 2: Verify
[[ -n "$STAGED_GO" ]] && golangci-lint run ./...
[[ -n "$STAGED_TS" ]] && npx tsc --noEmit && npx eslint --max-warnings 0 $STAGED_TS
[[ -n "$STAGED_RS" ]] && cargo clippy -- -D warnings
[[ -n "$STAGED_PY" ]] && ruff check $STAGED_PY && mypy $STAGED_PY

# File length check
for f in $STAGED_GO $STAGED_TS $STAGED_RS $STAGED_PY; do
  [[ -f "$f" ]] && lines=$(wc -l < "$f") && [[ $lines -gt 500 ]] && echo "ERROR: $f ($lines > 500 lines)" && exit 1
done
```

Only include sections for detected languages. Make executable:
```bash
chmod +x .git/hooks/pre-commit
```

#### update (modify existing)

1. Read `.git/hooks/pre-commit`
2. Compare against the tool matrix for detected languages
3. Add missing checks (formatters, linters, type checks, length checks)
4. Preserve any custom project-specific checks already in the hook

#### check (audit)

1. Read existing hook
2. Report what's configured vs what's missing
3. Time the hook execution on a sample commit to check if it's under 10s

#### fix (run fixes now)

Run auto-fix tools on all files (not just staged):

```bash
# Go
[[ -f go.mod ]] && gofmt -w . && goimports -w .

# TypeScript
[[ -f package.json ]] && npx prettier --write . && npx eslint --fix .

# Rust
[[ -f Cargo.toml ]] && cargo fmt

# Python
[[ -f pyproject.toml ]] && ruff format . && ruff check --fix .
```

### 5. Test the hook

Stage a file and run the hook manually:
```bash
git stash
echo "// test" >> <some-file>
git add <some-file>
.git/hooks/pre-commit
git checkout -- <some-file>
git stash pop
```

### 6. Report

```
## Quality Gates: [mode]

**Languages:** [detected]
**Hook:** [created|updated|exists|missing]

### Checks Configured
| Check | Tool | Auto-fix | Phase |
|-------|------|----------|-------|
| Formatting | [tool] | YES | 1 |
| Linting | [tool] | PARTIAL | 1+2 |
| Types | [tool] | NO | 2 |
| File length | wc -l | NO | 2 |

### Missing (if any)
- [check]: [tool needed]
```

**Next:** `/sdlc-status`

---

## Rules

- KEEP hook under 10 seconds — if it's slow, it gets disabled
- CHECK staged files only (not whole repo) in pre-commit
- AUTO-FIX first, verify second
- RE-STAGE fixed files after auto-fix
- FAIL with context — show file:line and how to fix
- Only include checks for detected languages — don't install Go checks in a Rust project
"#;

const SDLC_SETUP_QUALITY_GATES_PLAYBOOK: &str = r#"# sdlc-setup-quality-gates

Use this playbook to set up pre-commit hooks for a project.

> Read `.sdlc/guidance.md` (§6: never edit YAML directly). <!-- sdlc:guidance -->

## Modes

- `setup` — detect languages, create `.git/hooks/pre-commit`
- `update` — read existing hook, add missing checks
- `check` — audit existing hook, report gaps
- `fix` — run all auto-fix tools on the entire codebase

## Steps

1. Detect languages: check for `go.mod`, `Cargo.toml`, `package.json`, `pyproject.toml`.
2. Check existing hooks: `cat .git/hooks/pre-commit`.
3. For `setup`: create a two-phase pre-commit hook:
   - Phase 1: Auto-fix (formatters, `--fix` linters, re-stage)
   - Phase 2: Verify (lint, type check, file length ≤500 lines)
4. For `update`: read existing hook, compare against tool matrix, add missing checks.
5. For `check`: audit hook, report configured vs missing checks.
6. For `fix`: run auto-fix tools on all files (not staged-only).
7. Test: stage a file, run hook manually, verify it passes.

## Tool Matrix

| Check | Go | TypeScript | Rust | Python |
|-------|-----|------------|------|--------|
| Format | gofmt | prettier | rustfmt | black/ruff |
| Lint | golangci-lint | eslint | clippy | ruff |
| Types | compiler | tsc | compiler | mypy |

## Key Rules

- Keep hook under 10 seconds
- Check staged files only in pre-commit
- Auto-fix first, verify second
- Only include checks for detected languages
"#;

const SDLC_SETUP_QUALITY_GATES_SKILL: &str = r#"---
name: sdlc-setup-quality-gates
description: Set up pre-commit hooks and quality gates for a project. Use when configuring automated quality enforcement.
---

# SDLC Setup Quality Gates Skill

Use this skill to set up pre-commit hooks and quality gates for a project.

> Read `.sdlc/guidance.md` (§6: never edit YAML directly). <!-- sdlc:guidance -->

## Workflow

1. Detect project languages (Go, TypeScript, Rust, Python).
2. Create or update `.git/hooks/pre-commit` with two-phase approach: auto-fix then verify.
3. Wire quality gates into `.sdlc/config.yaml` for agent enforcement.
4. Test the hook with a sample staged file.
5. Report: checks configured, missing gaps, sdlc config status.

## Key Rules

- Hook must run under 10 seconds (staged files only).
- Auto-fix phase runs formatters and `--fix` linters, then re-stages.
- Verify phase runs lint, type check, and file length check.
- Only include tools for detected languages.
"#;

pub static SDLC_SETUP_QUALITY_GATES: CommandDef = CommandDef {
    slug: "sdlc-setup-quality-gates",
    claude_content: SDLC_SETUP_QUALITY_GATES_COMMAND,
    gemini_description: "Set up pre-commit hooks and quality gates for this project",
    playbook: SDLC_SETUP_QUALITY_GATES_PLAYBOOK,
    opencode_description: "Set up pre-commit hooks and quality gates for this project",
    opencode_hint: "<setup|update|check|fix>",
    skill: SDLC_SETUP_QUALITY_GATES_SKILL,
};
