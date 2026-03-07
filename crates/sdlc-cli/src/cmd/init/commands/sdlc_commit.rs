use crate::cmd::init::registry::CommandDef;

const SDLC_COMMIT_COMMAND: &str = r#"---
description: Commit changes to main with safe upstream merge — stages, commits, fetches origin, and reconciles diverged history without pushing
argument-hint: [commit message]
allowed-tools: Bash, Read, Grep, Glob
---

# sdlc-commit

Commit all current changes to `main`, reconcile with `origin/main` if diverged, never push.

> **Before acting:** read `.sdlc/guidance.md` for engineering principles. <!-- sdlc:guidance -->

## Usage

```
/sdlc-commit                → auto-generates commit message from diff
/sdlc-commit fix auth bug   → commit with explicit message
```

## Steps

### 1. Generate the commit message

If `$ARGUMENTS` is non-empty, use it as the commit message. Skip to Step 2.

If `$ARGUMENTS` is empty, generate a message:

```bash
git diff HEAD --stat
git diff HEAD
```

Read the diff output. Write a single-line commit message (120 chars max) that describes
**what changed and why**, not which files were touched. Examples of good messages:

- `feat: add ponder delete command with CLI, server endpoint, and frontend button`
- `fix: prevent duplicate SSE connections when navigating between ponder entries`
- `refactor: move commit message generation from Rust CLI to agent skill template`

Use conventional-commit style prefixes: `feat:`, `fix:`, `refactor:`, `docs:`, `chore:`, `test:`.
Focus on the semantic meaning of the change, not the mechanical diff.

### 2. Run the commit command

```bash
sdlc commit --message "<generated or provided message>"
```

If `--json` output reports `"conflict": true`, help the user resolve:
- Show the conflicting files
- Explain: resolve conflicts, then `git merge --continue && git branch -d dev/xist`

### 3. Report result

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

1. If no message provided: run `git diff HEAD --stat` and `git diff HEAD`, then write a 120-char-max commit message describing **what changed and why** (use `feat:`/`fix:`/`refactor:`/`docs:`/`chore:` prefixes)
2. Run: `sdlc commit --message "<message>"`
3. If conflict reported: show files, instruct user to resolve then `git merge --continue && git branch -d dev/xist`
4. Report commit SHA, merge status, ahead/behind count
5. Remind: not pushed — `git push origin main` when ready
"#;

const SDLC_COMMIT_SKILL: &str = r#"---
name: sdlc-commit
description: Commit changes to main with safe upstream merge — stages, commits, fetches origin, and reconciles diverged history without pushing
---

# SDLC Commit Skill

Commit all changes to main and reconcile with origin/main if it has diverged.

> Read `.sdlc/guidance.md` (§6: never edit YAML directly). <!-- sdlc:guidance -->

## Workflow

1. If no message provided: run `git diff HEAD --stat` and `git diff HEAD`, then write a 120-char-max commit message describing what changed and why (use feat:/fix:/refactor:/docs:/chore: prefixes)
2. Run: `sdlc commit --message "<message>"`
3. If conflict: show files, instruct to resolve then `git merge --continue && git branch -d dev/xist`
4. Report result: commit SHA, merge status, ahead/behind
5. Never pushes — remind user to `git push origin main`
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
