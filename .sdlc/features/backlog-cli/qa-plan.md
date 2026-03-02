# QA Plan: sdlc backlog CLI commands

## Scope

Verify the `sdlc backlog` CLI command group works end-to-end: add, list, park, promote, show subcommands with all flags, error paths, auto-inference, and JSON output.

## Test execution

```bash
SDLC_NO_NPM=1 cargo test --all
```

All tests run in isolated `tempfile::TempDir` environments. No real `.sdlc/` directory is touched.

## Test Cases

### TC-1: add — basic confirmation output format

**Given:** Initialized project with an active feature `my-feature` in state.yaml  
**When:** `sdlc backlog add auth race condition --kind concern`  
**Then:**
- Exit code 0
- Stdout contains: `Backlog item B1 recorded: "auth race condition" [my-feature]`
- `.sdlc/backlog.yaml` has one item with `id: B1`, `status: open`, `kind: concern`, `source_feature: my-feature`

### TC-2: add — explicit --source-feature

**Given:** Initialized project  
**When:** `sdlc backlog add refactor db layer --kind debt --source-feature other-feature`  
**Then:**
- Stdout contains: `Backlog item B1 recorded: "refactor db layer" [other-feature]`
- `source_feature` in YAML is `other-feature`

### TC-3: add — no active feature warning

**Given:** Project with empty `active_features` in state.yaml  
**When:** `sdlc backlog add stale index --kind debt`  
**Then:**
- Exit code 0
- Stderr contains: `warning: no active feature found`
- Stdout contains: `Backlog item B1 recorded: "stale index" [none]`
- `source_feature` is absent from YAML

### TC-4: add — all optional fields

**Given:** Initialized project  
**When:** `sdlc backlog add slow query --kind concern --description "hits full scan" --evidence "src/db.rs:42" --source-feature db-opt`  
**Then:**
- All fields present in `.sdlc/backlog.yaml`

### TC-5: add — invalid kind

**When:** `sdlc backlog add title --kind badkind`  
**Then:** Exit code non-zero; stderr contains `unknown kind 'badkind'`

### TC-6: list — defaults to open items

**Given:** B1 (open), B2 (parked)  
**When:** `sdlc backlog list`  
**Then:** Output contains B1, does not contain B2

### TC-7: list --all

**Given:** B1 (open), B2 (parked), B3 (promoted)  
**When:** `sdlc backlog list --all`  
**Then:** Output contains B1, B2, B3

### TC-8: list --status parked

**Given:** B1 (open), B2 (parked)  
**When:** `sdlc backlog list --status parked`  
**Then:** Output contains B2, does not contain B1

### TC-9: list --source-feature filter

**Given:** B1 (source: feat-a), B2 (source: feat-b)  
**When:** `sdlc backlog list --source-feature feat-a`  
**Then:** Output contains B1, does not contain B2

### TC-10: list --all --status conflict

**When:** `sdlc backlog list --all --status open`  
**Then:** Exit code non-zero; clap conflict error

### TC-11: park — success

**Given:** B1 (open)  
**When:** `sdlc backlog park B1 --reason revisit after v14`  
**Then:**
- Exit code 0
- Stdout: `Parked B1: revisit after v14`
- B1 status in YAML is `parked`, `park_reason` is `"revisit after v14"`

### TC-12: park — missing --reason

**When:** `sdlc backlog park B1`  
**Then:** Exit code non-zero; clap error indicating `--reason` is required

### TC-13: park — unknown ID

**When:** `sdlc backlog park B99 --reason test`  
**Then:** Exit code non-zero; stderr contains `B99`

### TC-14: park — already promoted

**Given:** B1 promoted  
**When:** `sdlc backlog park B1 --reason test`  
**Then:** Exit code non-zero; error about cannot park promoted item

### TC-15: promote — auto-slug from title

**Given:** B1 with title `"Fix auth token race"`  
**When:** `sdlc backlog promote B1`  
**Then:**
- Feature `fix-auth-token-race` created in `.sdlc/features/`
- B1 status is `promoted`, `promoted_to` is `fix-auth-token-race`
- Stdout: `Promoted B1 → feature: fix-auth-token-race`
- Feature appears in `state.yaml` `active_features`

### TC-16: promote — explicit --slug

**Given:** B1 with any title  
**When:** `sdlc backlog promote B1 --slug my-custom-slug`  
**Then:**
- Feature `my-custom-slug` is created
- B1 `promoted_to` is `my-custom-slug`

### TC-17: promote — with --milestone

**Given:** B1 (open), milestone `v10` exists  
**When:** `sdlc backlog promote B1 --milestone v10`  
**Then:**
- Feature created and linked in milestone `v10`
- Stdout contains `Added to milestone: v10`

### TC-18: promote — already promoted

**Given:** B1 already promoted  
**When:** `sdlc backlog promote B1`  
**Then:** Exit code non-zero; error about already promoted

### TC-19: show — human output

**Given:** B1 with all fields set  
**When:** `sdlc backlog show B1`  
**Then:** All non-None fields printed; `park_reason` absent when status is open

### TC-20: show — unknown ID

**When:** `sdlc backlog show B99`  
**Then:** Exit code non-zero; stderr contains `B99`

### TC-21: JSON output — add

**When:** `sdlc backlog add title --json`  
**Then:** Stdout is valid JSON object with `id`, `title`, `kind`, `status` keys

### TC-22: JSON output — list

**When:** `sdlc backlog list --json`  
**Then:** Stdout is valid JSON array

### TC-23: JSON output — park

**When:** `sdlc backlog park B1 --reason test --json`  
**Then:** Stdout is valid JSON with `status: "parked"`

### TC-24: JSON output — promote

**When:** `sdlc backlog promote B1 --json`  
**Then:** Stdout is valid JSON with `status: "promoted"` and `promoted_to` set

### TC-25: JSON output — show

**When:** `sdlc backlog show B1 --json`  
**Then:** Stdout is valid JSON object

### TC-26: slugify helper — truncation

**Unit test:** `slugify("a very long title that exceeds forty characters limit here")` returns a slug ≤ 40 chars that does not end with a dash

### TC-27: slugify helper — special characters

**Unit test:** `slugify("Fix auth.rs: token race! (critical)")` returns `fix-auth-rs-token-race-critical`

### TC-28: build passes

**When:** `cargo build --all`  
**Then:** No errors, no warnings (or only pre-existing warnings)

### TC-29: clippy passes

**When:** `cargo clippy --all -- -D warnings`  
**Then:** No new clippy violations introduced by this feature

## Pass criteria

All 29 test cases pass. `SDLC_NO_NPM=1 cargo test --all` exits 0. `cargo clippy --all -- -D warnings` exits 0.
