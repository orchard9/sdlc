# QA Plan: /sdlc-knowledge Slash Command

## Approach

This is a pure Rust const/static addition with no runtime logic or server changes. QA is compilation, lint, and structural inspection.

## Test Cases

### QC1 — Build succeeds

```bash
SDLC_NO_NPM=1 cargo build --all
```

Expected: exits 0, no errors.

### QC2 — All tests pass

```bash
SDLC_NO_NPM=1 cargo test --all
```

Expected: exits 0, no test failures.

### QC3 — Clippy clean

```bash
cargo clippy --all -- -D warnings
```

Expected: exits 0, no warnings.

### QC4 — Command file is syntactically valid Rust

Covered by QC1. The file must compile without errors. The `CommandDef` fields must match the struct definition in `registry.rs`.

### QC5 — `ALL_COMMANDS` includes the new entry

After implementation, grep for `sdlc_knowledge` in `commands/mod.rs`:

```bash
grep sdlc_knowledge crates/sdlc-cli/src/cmd/init/commands/mod.rs
```

Expected: at least two lines — the `mod` declaration and the `ALL_COMMANDS` entry.

### QC6 — Command file has all three required constants

```bash
grep -E "SDLC_KNOWLEDGE_COMMAND|SDLC_KNOWLEDGE_PLAYBOOK|SDLC_KNOWLEDGE_SKILL|pub static SDLC_KNOWLEDGE" \
  crates/sdlc-cli/src/cmd/init/commands/sdlc_knowledge.rs
```

Expected: four matches.

### QC7 — Section 6 of GUIDANCE_MD_CONTENT contains knowledge commands

```bash
grep "sdlc knowledge" crates/sdlc-cli/src/cmd/init/templates.rs
```

Expected: at least 7 lines covering `status`, `list`, `search`, `show`, `add`, `catalog show`, `librarian init`.

### QC8 — Command content covers all five modes

Check that `SDLC_KNOWLEDGE_COMMAND` covers the five dispatch modes:

```bash
grep -E "no arg|init|research|maintain|query|topic" \
  crates/sdlc-cli/src/cmd/init/commands/sdlc_knowledge.rs
```

Expected: evidence of all five modes in the command content.

### QC9 — `sdlc update` installs the command file (manual spot-check)

After building, run:

```bash
./target/debug/sdlc update 2>&1 | grep knowledge
```

Expected: output lines showing `created` or `updated` for `sdlc-knowledge.md`, `sdlc-knowledge.toml`, and `sdlc-knowledge/SKILL.md`.
