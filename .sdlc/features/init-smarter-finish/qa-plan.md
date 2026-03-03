# QA Plan: Smarter sdlc-init Finish

## Scope

This QA plan covers verification of the updated `SDLC_INIT_COMMAND`, `SDLC_INIT_PLAYBOOK`, and `SDLC_INIT_SKILL` constants in `crates/sdlc-cli/src/cmd/init/commands/sdlc_init.rs`. Since this is a text-only change to embedded string constants, QA focuses on:

1. Correct content of the Phase 7 section (assess → synthesize → seed)
2. Correct content updates to PLAYBOOK and SKILL variants
3. Successful compilation
4. Successful `sdlc update` installation of the updated templates

---

## TC-1: Phase 7 Section Presence in SDLC_INIT_COMMAND

**Check:** The `SDLC_INIT_COMMAND` constant contains a Phase 7 section covering:
- Phase 7a: scope thickness assessment (thin vs. thick criteria)
- Phase 7b: milestone structure derivation rules (slug, title, vision, features, acceptance test)
- Phase 7c: seed CLI commands (`sdlc milestone create`, `sdlc milestone update`, `sdlc milestone set-acceptance-test`, `sdlc feature create`, `sdlc milestone add-feature`)

**Pass:** All three sub-phases are present with the correct CLI commands.
**Fail:** Any sub-phase missing or incorrect CLI command.

---

## TC-2: Updated Finish Summary Block

**Check:** The Finish summary block in `SDLC_INIT_COMMAND` includes:
```
✓ First milestone seeded: <slug> ([N] features)
```

**Pass:** The seeded milestone line is present in the Finish block.
**Fail:** Finish block still shows the old generic text without the seeded milestone line.

---

## TC-3: Updated Next Line (Thick Scope Path)

**Check:** The `**Next:**` line at the end of `SDLC_INIT_COMMAND` reads:
```
**Next:** /sdlc-prepare <slug>
```
(not the old `/sdlc-ponder or /sdlc-plan`)

**Pass:** Next line references `/sdlc-prepare`.
**Fail:** Next line still references only `/sdlc-ponder` or `/sdlc-plan` without the milestone path.

---

## TC-4: Thin Scope Fallback Path Present

**Check:** The `SDLC_INIT_COMMAND` Phase 7a section explicitly documents the thin-scope skip path that falls back to:
```
**Next:** /sdlc-ponder
```

**Pass:** Thin scope fallback is documented in Phase 7a or the Next section.
**Fail:** No thin scope fallback documented.

---

## TC-5: SDLC_INIT_PLAYBOOK Updated

**Check:** The `SDLC_INIT_PLAYBOOK` constant has a step 11 covering milestone seeding, and its final **Next** includes the milestone-focused path.

**Pass:** Step 11 present, Next includes milestone path.
**Fail:** Playbook not updated.

---

## TC-6: SDLC_INIT_SKILL Updated

**Check:** The `SDLC_INIT_SKILL` constant has an updated Outcome table with:
- Scope thick → `/sdlc-prepare <slug>`
- Scope thin → `/sdlc-ponder`

**Pass:** Both outcomes in the table.
**Fail:** Outcome table not updated.

---

## TC-7: Compilation Passes

**Command:**
```bash
SDLC_NO_NPM=1 cargo build -p sdlc-cli 2>&1
```

**Pass:** Zero compilation errors.
**Fail:** Any error.

---

## TC-8: Idempotency Note Present

**Check:** Phase 7c or Phase 7b in `SDLC_INIT_COMMAND` contains a note that the seed is idempotent — re-running sdlc-init updates existing milestones/features rather than creating duplicates.

**Pass:** Idempotency note present.
**Fail:** No idempotency note.

---

## TC-9: `sdlc update` Installs Updated Template

**Command:**
```bash
cargo run -p sdlc-cli -- update
grep -l "sdlc-prepare" ~/.claude/commands/sdlc-init.md
```

**Pass:** `sdlc-init.md` in `~/.claude/commands/` contains `/sdlc-prepare`.
**Fail:** File not updated or does not contain the new Next line.

---

## Pass Criteria

All 9 test cases must pass for the feature to advance to merge.
