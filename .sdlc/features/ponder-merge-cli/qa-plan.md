# QA Plan: sdlc ponder merge — CLI command and core data model

## QA-1: Successful merge end-to-end

**Steps:**
1. Create two ponder entries: `sdlc ponder create qa-src --title "Source"` and `sdlc ponder create qa-tgt --title "Target"`
2. Add a session to source: `sdlc ponder session log qa-src --content "session content"`
3. Add an artifact to source: `sdlc ponder capture qa-src --content "artifact body" --as notes.md`
4. Add a team member to source: `sdlc ponder team add qa-src --name "test-partner" --role "Tester" --context "QA" --agent ".claude/agents/test.md"`
5. Run: `sdlc ponder merge qa-src --into qa-tgt --json`

**Expected:**
- JSON output contains `sessions_copied: 1`, `artifacts_copied: 1`, `team_members_copied: 1`
- `sdlc ponder show qa-src --json` has `merged_into: "qa-tgt"` and `status: "parked"`
- `sdlc ponder show qa-tgt --json` has `merged_from: ["qa-src"]` and session count incremented
- Target sessions dir contains the copied session with merge header comment
- Target dir contains `notes.md` artifact

## QA-2: Merged entries hidden from default list

**Steps:**
1. After QA-1, run `sdlc ponder list --json`
2. Run `sdlc ponder list --all --json`

**Expected:**
- Default list does NOT include `qa-src`
- `--all` list includes `qa-src` with `merged_into` field present

## QA-3: Show redirect banner

**Steps:**
1. After QA-1, run `sdlc ponder show qa-src` (non-JSON)

**Expected:**
- Output contains "merged into 'qa-tgt'"

## QA-4: Reject merge of committed source

**Steps:**
1. Create entry: `sdlc ponder create qa-committed --title "Committed"`
2. Mark committed: `sdlc ponder update qa-committed --status committed`
3. Create target: `sdlc ponder create qa-tgt2 --title "Target 2"`
4. Run: `sdlc ponder merge qa-committed --into qa-tgt2`

**Expected:**
- Command fails with error containing "cannot merge committed entry"

## QA-5: Reject merge into committed target

**Steps:**
1. Create source: `sdlc ponder create qa-src2 --title "Source 2"`
2. Create target: `sdlc ponder create qa-committed-tgt --title "Committed Target"`
3. Mark target committed: `sdlc ponder update qa-committed-tgt --status committed`
4. Run: `sdlc ponder merge qa-src2 --into qa-committed-tgt`

**Expected:**
- Command fails with error containing "cannot merge into committed entry"

## QA-6: Reject self-merge

**Steps:**
1. Create entry: `sdlc ponder create qa-self --title "Self"`
2. Run: `sdlc ponder merge qa-self --into qa-self`

**Expected:**
- Command fails with error containing "cannot merge an entry into itself"

## QA-7: Artifact collision prefix

**Steps:**
1. Create source and target entries
2. Add `notes.md` artifact to both source and target
3. Run merge

**Expected:**
- Target dir contains both `notes.md` (original) and `<source-slug>--notes.md` (from source)

## QA-8: Serde backward compatibility

**Steps:**
1. Verify existing ponder entries without `merged_into`/`merged_from` fields load without error
2. Run `sdlc ponder list` on existing project ponder entries

**Expected:**
- All existing entries load cleanly — new fields default to None/empty

## QA-9: Build and test suite

**Steps:**
1. `SDLC_NO_NPM=1 cargo build --all`
2. `SDLC_NO_NPM=1 cargo test --all`
3. `cargo clippy --all -- -D warnings`

**Expected:**
- All pass with zero errors, zero warnings
