use crate::cmd::init::registry::CommandDef;

const SDLC_QUALITY_FIX_COMMAND: &str = r#"---
description: Fix failing quality-check results — reads /tmp/quality-check-result.json and applies the right fix strategy
argument-hint: [tool-name]
allowed-tools: Bash, Read, Write, Edit, Glob, Grep
---

# sdlc-quality-fix

Fix failing quality-check results. Reads `/tmp/quality-check-result.json` (written automatically by the quality-check tool when checks fail), selects the right fix strategy by failure count, and applies it.

> **Before acting:** read `.sdlc/guidance.md`. <!-- sdlc:guidance -->

## Steps

### 1. Load failure data

```bash
cat /tmp/quality-check-result.json | jq '{ok, "failed": .data.failed, checks: [.data.checks[] | select(.status=="failed") | {name, output}]}'
```

If the file doesn't exist, run the quality-check tool first:
```bash
sdlc tool run quality-check
```

### 2. Triage by failure count

| Failures | Strategy | Rationale |
|----------|----------|-----------|
| 1 | fix-forward | Single targeted fix — patch, then confirm it's really fixed |
| 2–5 | fix-all | Multi-dimension review + fix across all seven code health axes |
| 6+ | remediate | Systemic problem — root-cause, enforce, document, verify |

### 3. Apply strategy

**1 failure → fix-forward:**
- Read the failing check name and its output from the JSON
- Diagnose: is this a one-line fix or a structural problem?
- If fixable: apply the minimal correct fix, re-run `sdlc tool run quality-check` to verify
- If structural: invoke `/fix-forward` with the check name as context

**2–5 failures → fix-all:**
- Extract all failing check names and their output
- Invoke `/fix-all` scoped to the files the failing checks touched
- Re-run `sdlc tool run quality-check` after fixes

**6+ failures → remediate:**
- The check suite is revealing a systemic issue
- Invoke `/remediate` with context: "quality-check found <N> failures: <check names>"
- The remediate skill will root-cause, fix, enforce, document, and verify

### 4. Verify

```bash
sdlc tool run quality-check
```

Expected: all previously failing checks now pass. If new failures appear, re-triage from Step 2.

**Next:** `/sdlc-setup-quality-gates update` if hook coverage is incomplete
"#;

const SDLC_QUALITY_FIX_PLAYBOOK: &str = r#"# sdlc-quality-fix

Fix failing quality-check results. Reads `/tmp/quality-check-result.json` (written by the quality-check tool when checks fail), selects the right fix strategy, and applies it.

> Read `.sdlc/guidance.md`. <!-- sdlc:guidance -->

## Steps

### 1. Load failure data
`cat /tmp/quality-check-result.json | jq '{ok, "failed": .data.failed, checks: [.data.checks[] | select(.status=="failed") | {name, output}]}'`

If the file doesn't exist, run quality-check first:
`sdlc tool run quality-check`

### 2. Triage by failure count

| Failures | Strategy |
|----------|----------|
| 1 | Targeted patch — diagnose, fix, verify |
| 2–5 | Multi-dimension fix across all affected code |
| 6+ | Root-cause investigation, enforce, document |

### 3. Apply strategy

Extract failing check names and outputs. For each:
- Read the check output to understand the root cause
- Apply the minimal correct fix
- Avoid patching symptoms — fix the underlying issue

### 4. Verify
`sdlc tool run quality-check`
Expected: all previously failing checks now pass.

**Next:** `sdlc tool run quality-check` to verify
"#;

const SDLC_QUALITY_FIX_SKILL: &str = r#"---
name: sdlc-quality-fix
description: Fix failing quality-check results — load /tmp/quality-check-result.json, triage by failure count, apply the right fix strategy, and verify. Use when quality-check reports failures.
---

# SDLC Quality-Fix Skill

Fix failing quality-check results.

> Read `.sdlc/guidance.md`. <!-- sdlc:guidance -->

## Workflow

1. `cat /tmp/quality-check-result.json | jq '{ok, failed: .data.failed}'` — load failure data
2. Triage: 1 failure → targeted patch; 2–5 → multi-fix; 6+ → root-cause + remediate
3. Fix each failing check by reading its `output` field and applying the correct change
4. `sdlc tool run quality-check` — verify all checks now pass

End: `**Next:** sdlc tool run quality-check` to confirm clean.
"#;

pub static SDLC_QUALITY_FIX: CommandDef = CommandDef {
    slug: "sdlc-quality-fix",
    claude_content: SDLC_QUALITY_FIX_COMMAND,
    gemini_description: "Fix failing quality-check results by triage and targeted fix",
    playbook: SDLC_QUALITY_FIX_PLAYBOOK,
    opencode_description: "Fix failing quality-check results by triage and targeted fix",
    opencode_hint: "[tool-name]",
    skill: SDLC_QUALITY_FIX_SKILL,
};
