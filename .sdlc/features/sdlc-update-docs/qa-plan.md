# QA Plan: Document sdlc update as Update Mechanism

## Scope

Two file changes:
1. `README.md` — new "Updating" section
2. `crates/sdlc-cli/src/cmd/init/mod.rs` — init completion message

## Test Cases

### QA-1: README contains Updating section

**Steps:**
1. Open `README.md`
2. Search for `## Updating`

**Expected:** Section exists, contains `sdlc update` command, mentions `~/.claude/commands/`, and explains the purpose of running after every upgrade.

---

### QA-2: README section is placed correctly

**Steps:**
1. Locate the `## Install` section in `README.md`
2. Verify `## Updating` appears immediately after it (before the next major section)

**Expected:** Logical reading order — Install → Updating.

---

### QA-3: init completion message is correct

**Steps:**
1. Open `crates/sdlc-cli/src/cmd/init/mod.rs`
2. Search for the "Next:" println in the init completion block

**Expected:** Message reads `Next: sdlc ui    # then visit /setup to define Vision and Architecture` (not the old `sdlc feature create` message).

---

### QA-4: Build succeeds

**Steps:**
```bash
SDLC_NO_NPM=1 cargo build --all
```

**Expected:** Clean build with no errors or warnings that weren't present before.

---

### QA-5: Clippy passes

**Steps:**
```bash
SDLC_NO_NPM=1 cargo clippy --all -- -D warnings
```

**Expected:** Zero new warnings or errors.

---

### QA-6: Tests pass

**Steps:**
```bash
SDLC_NO_NPM=1 cargo test --all
```

**Expected:** All tests pass (no new failures).
