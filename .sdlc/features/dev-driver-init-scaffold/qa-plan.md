# QA Plan: dev-driver scaffolded by sdlc init and sdlc update

## Scope

Verify that `sdlc init` and `sdlc update` correctly scaffold the `dev-driver` tool into any
project's `.sdlc/tools/` directory with correct write policies.

## Test Cases

### TC-1 — Fresh init produces dev-driver files

**Steps:**
1. Create a temp directory with no `.sdlc/` directory.
2. Run `sdlc init` (or call `write_core_tools` in a unit test against a TempDir).
3. Assert `.sdlc/tools/dev-driver/tool.ts` exists and is non-empty.
4. Assert `.sdlc/tools/dev-driver/README.md` exists and is non-empty.

**Expected:** Both files present and match `TOOL_DEV_DRIVER_TS` / `TOOL_DEV_DRIVER_README_MD` constants.

---

### TC-2 — tool.ts is overwritten on re-run

**Steps:**
1. After TC-1, write arbitrary content to `.sdlc/tools/dev-driver/tool.ts`.
2. Run `write_core_tools` again.
3. Assert `tool.ts` content matches `TOOL_DEV_DRIVER_TS` (overwritten).

**Expected:** `tool.ts` contains the canonical content, not the arbitrary content.

---

### TC-3 — README.md is not overwritten on re-run

**Steps:**
1. After TC-1, write arbitrary content to `.sdlc/tools/dev-driver/README.md`.
2. Run `write_core_tools` again.
3. Assert `README.md` content is unchanged (still the arbitrary content).

**Expected:** `README.md` retains user content — write-if-missing policy is honoured.

---

### TC-4 — tools.md includes dev-driver entry

**Steps:**
1. After `write_core_tools` in TC-1, read `.sdlc/tools/tools.md`.
2. Assert the file contains the string `dev-driver`.

**Expected:** Static tools.md has a `dev-driver` section.

---

### TC-5 — Build compiles clean

**Steps:**
1. Run `SDLC_NO_NPM=1 cargo build --all`.
2. Assert exit code 0.

**Expected:** No compile errors. `include_str!` paths resolve correctly.

---

### TC-6 — Clippy passes

**Steps:**
1. Run `cargo clippy --all -- -D warnings`.
2. Assert exit code 0.

**Expected:** No new warnings introduced.

---

### TC-7 — sdlc tool list shows dev-driver (manual smoke test)

**Steps:**
1. Run `sdlc init` in a fresh temp directory.
2. Run `sdlc tool list`.
3. Assert `dev-driver` appears in the output.

**Expected:** `dev-driver` listed as an available tool.

## Pass Criteria

All 7 test cases pass. TC-1 through TC-6 can be verified in CI; TC-7 is a manual smoke test.
