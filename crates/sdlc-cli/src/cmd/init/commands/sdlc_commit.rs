use crate::cmd::init::registry::CommandDef;

const SDLC_COMMIT_COMMAND: &str = r#"---
description: Commit changes to main with safe upstream merge — stages, commits, fetches origin, and reconciles diverged history without pushing
argument-hint: [commit message]
allowed-tools: Bash
---

# sdlc-commit

Commit all current changes to `main`, reconcile with `origin/main` if diverged, never push.

> **Before acting:** read `.sdlc/guidance.md` for engineering principles. <!-- sdlc:guidance -->

## Usage

```
/sdlc-commit                → auto-generates message from diff summary
/sdlc-commit fix auth bug   → commit with explicit message
```

## Steps

### 1. Run the commit command

```bash
sdlc commit --message "$ARGUMENTS"
```

If `$ARGUMENTS` is empty, omit `--message` (auto-generates a brief description from the diff).

If `--json` output reports `"conflict": true`, help the user resolve:
- Show the conflicting files
- Explain: resolve conflicts, then `git merge --continue && git branch -d dev/xist`

### 2. Report result

Show what happened:
- The commit SHA and message
- Whether an upstream merge was needed
- The ahead/behind status
- Remind: **not pushed** — run `git push origin main` when ready

**Next:** `git push origin main` — push when ready
"#;

const SDLC_COMMIT_PLAYBOOK: &str = r#"# sdlc-commit

Commit changes to main, reconcile with origin/main if diverged, never push.

> Read `.sdlc/guidance.md` (§6: never edit YAML directly). <!-- sdlc:guidance -->

## Steps

1. Run: `sdlc commit --message "<message>"` (omit --message to auto-generate from diff)
2. If conflict reported: show files, instruct user to resolve then `git merge --continue && git branch -d dev/xist`
3. Report commit SHA, merge status, ahead/behind count
4. Remind: not pushed — `git push origin main` when ready
"#;

const SDLC_COMMIT_SKILL: &str = r#"---
name: sdlc-commit
description: Commit changes to main with safe upstream merge — stages, commits, fetches origin, and reconciles diverged history without pushing
---

# SDLC Commit Skill

Commit all changes to main and reconcile with origin/main if it has diverged.

> Read `.sdlc/guidance.md` (§6: never edit YAML directly). <!-- sdlc:guidance -->

## Workflow

1. Run: `sdlc commit --message "<message>"` (omit --message to auto-generate from diff)
2. If conflict: show files, instruct to resolve then `git merge --continue && git branch -d dev/xist`
3. Report result: commit SHA, merge status, ahead/behind
4. Never pushes — remind user to `git push origin main`
"#;

pub static SDLC_COMMIT: CommandDef = CommandDef {
    slug: "sdlc-commit",
    claude_content: SDLC_COMMIT_COMMAND,
    gemini_description: "Commit changes to main with safe upstream merge",
    playbook: SDLC_COMMIT_PLAYBOOK,
    opencode_description: "Commit changes to main with safe upstream merge — stages, commits, fetches origin, reconciles diverged history",
    opencode_hint: "[commit message]",
    skill: SDLC_COMMIT_SKILL,
};
