# Design: UAT Failure Triage and 3 Pathways

## Overview

This is a pure template/CLI change — no UI, no server routes, no database migrations. All changes live in `crates/sdlc-cli/src/cmd/init/commands/sdlc_milestone_uat.rs` and a new `sdlc_recap.rs` command file.

## Architecture

### Affected Files

| File | Change |
|---|---|
| `crates/sdlc-cli/src/cmd/init/commands/sdlc_milestone_uat.rs` | Replace Steps 5–6 with Steps 5–9 (triage + 3 pathways) |
| `crates/sdlc-cli/src/cmd/init/commands/sdlc_recap.rs` | New file: `/sdlc-recap` command (4 platform variants) |
| `crates/sdlc-cli/src/cmd/init/commands/mod.rs` | Add `mod sdlc_recap;` |
| `crates/sdlc-cli/src/cmd/init/registry.rs` | `ALL_COMMANDS` already auto-picks from `mod.rs` — no change needed |
| `crates/sdlc-cli/src/cmd/init/mod.rs` | `write_user_*` functions already iterate `ALL_COMMANDS` — register recap in `ALL_COMMANDS` |
| `crates/sdlc-cli/src/cmd/init/templates.rs` | Add `/sdlc-recap` row to `GUIDANCE_MD_CONTENT` command table (§6) |

### No Rust Struct Changes

These are instruction text templates. No new enums, no new struct fields, no CLI subcommands. The changes are purely Markdown/text constants embedded in Rust string literals.

## Design: Updated `sdlc-milestone-uat` Template

### Step Structure (before → after)

**Before:**
- Step 1: Load milestone
- Step 2: Mode detection
- Mode A / Mode B tests
- Step 3: Write summary.md
- Step 4: Write uat_results.md
- Step 5: Flip milestone state
- Step 6: Final report

**After:**
- Step 1: Load milestone
- Step 2: Mode detection
- Mode A / Mode B tests
- Step 3: Write summary.md
- Step 4: Write uat_results.md
- Step 5: Triage failures (NEW)
- Step 6: Pathway 1 — Fix and Retry (NEW)
- Step 7: Pathway 2 — Escalate (NEW)
- Step 8: Pathway 3 — Recap and Propose (NEW)
- Step 9: Final report (updated table)

### Triage Classification Table

```
| Fixable    | Assertion fails on a value agent can change; route returns wrong status; missing CSS class |
| Escalation | Missing env var; server unreachable; unclear requirement; needs human judgment |
| Complex    | Wrong architectural approach; feature design doesn't match reality; multiple interacting failures |
```

### Pathway Decision Logic

```
ALL failures = Fixable        → Pathway 1 (fix + retry, max 2 cycles)
  After 2 cycles, still fail → reclassify remaining, fall through

ANY failure = Escalation
  AND NONE = Complex          → Pathway 2 (escalate)

ANY failure = Complex         → Pathway 3 (recap + ponder)
```

### Pathway 1: Fix and Retry

```
1. Fix code (< 3 files, targeted)
2. Rerun: npx playwright test e2e/milestones/<slug>.spec.ts --reporter=json
3. Re-parse results.json
4. Repeat up to 2 total cycles
5. If still failing: reclassify and fall through to Pathway 2 or 3
```

### Pathway 2: Escalate

```
1. sdlc task add <feature> "UAT: <description>" (for fixable items)
2. sdlc escalate create --kind <type> --title "UAT blocker: <desc>" \
     --context "<error context>" --feature <feature-slug>
3. curl -s -X POST http://localhost:7777/api/milestone/<slug>/uat/fail
4. **Next:** resolve escalation <id>, then /sdlc-milestone-uat <slug>
```

### Pathway 3: Recap and Propose

```
1. sdlc task add <feature> "UAT: <description>" (for fixable items)
2. sdlc status --json  (gather state)
3. For each complex failure:
     sdlc ponder create "<problem-as-question>" --brief "<context>"
4. git add -A && git commit -m "uat: partial progress on <slug>, ponder sessions proposed"
5. curl -s -X POST http://localhost:7777/api/milestone/<slug>/uat/fail
6. **Next:** /sdlc-ponder <first-ponder-slug>
```

### Updated Final Report Table

```
| Pass           | Released   | Commit summary.md and uat_results.md |
| PassWithTasks  | Released   | Commit results; /sdlc-run <feature> next cycle |
| FixedAndPassed | Released   | Commit results (retry succeeded) |
| Escalated      | Verifying  | Resolve escalation <id>, then /sdlc-milestone-uat <slug> |
| Recapped       | Verifying  | /sdlc-ponder <first-ponder-slug> |
```

## Design: `/sdlc-recap` Command

### Purpose

Standalone command to synthesize current project state and produce concrete forward-motion artifacts. Callable from UAT Pathway 3 or independently at any time.

### Command Flow

```
1. sdlc status --json              → gather active features, phases, tasks
2. sdlc milestone info <slug> --json  → (if milestone context exists)
3. git log --oneline -20           → recent git history
4. Synthesize into 4 sections:
   - Working On: features in implementation/review/qa
   - Completed: released features, recent milestones
   - Remaining: features in draft/specified/planned/ready
   - Forward Motion: concrete next artifacts
5. Forward Motion creates real artifacts:
   - sdlc task add <slug> "..." for incomplete work
   - sdlc escalate create for blockers
   - sdlc ponder create "<question>" --brief "..." for strategic questions
6. End with exactly one **Next:** line
```

### Platform Variants

- **Claude Code** (`sdlc-recap.md`): Full step-by-step with all 4 sections and Forward Motion creation
- **Gemini** (`sdlc-recap.toml`): Concise playbook variant
- **OpenCode** (`sdlc-recap.md`): Same as Claude format
- **Agent Skills** (`sdlc-recap/SKILL.md`): Minimal SKILL.md variant

### Registration

New `CommandDef` with slug `sdlc-recap`. Registered in `commands::ALL_COMMANDS` slice. The `write_user_*` functions iterate `ALL_COMMANDS` automatically — no per-function changes needed.

Add row to guidance table in `GUIDANCE_MD_CONTENT` (templates.rs):
```
| /sdlc-recap | Synthesize project state and create forward-motion artifacts |
```

Also add to `migrate_legacy_project_scaffolding` file lists (for cleanup of old project-level versions).

## Implementation Order

1. Update `sdlc_milestone_uat.rs` — replace Steps 5–6 with Steps 5–9 in all 3 variants
2. Create `sdlc_recap.rs` — 4 platform variants + `CommandDef`
3. Register in `mod.rs` and add to `ALL_COMMANDS` in `registry.rs`
4. Update `GUIDANCE_MD_CONTENT` in `templates.rs` — add `/sdlc-recap` row
5. Build and test: `SDLC_NO_NPM=1 cargo build --all && cargo clippy --all -- -D warnings`
6. Run `sdlc update` to install updated commands to user directories

## Quality Checks

- No `unwrap()` in any Rust code (these are string constants, no Rust logic)
- Clippy must pass: `cargo clippy --all -- -D warnings`
- `SDLC_NO_NPM=1 cargo test --all` must pass
- `sdlc update` installs `sdlc-recap` to all 4 platform dirs
