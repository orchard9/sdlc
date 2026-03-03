# QA Plan: changelog-cli

## Scope

Validate the `sdlc changelog` CLI command: flag parsing, event classification, pretty-print output, JSON output, edge cases (no runs, no matching runs, last-merge fallback).

## Test Cases

### TC-1 — Default output (no flags)

**Setup:** Project root with at least one run record in `.sdlc/.runs/` created within the last 7 days.

**Command:**
```
sdlc changelog
```

**Expected:**
- Exit 0
- Stdout contains at least one line with an icon (`⚠️`, `🚀`, `✅`, `🔄`, `▶`, or `⏹`)
- Output format: `<icon>  <label>  <relative-time>`
- Relative time ends in "ago"

---

### TC-2 — --since relative flag (1d)

**Setup:** Two run records: one created 30 minutes ago, one created 2 days ago.

**Command:**
```
sdlc changelog --since 1d --limit 10
```

**Expected:**
- Exit 0
- Only the run from 30 minutes ago appears
- Run from 2 days ago is excluded

---

### TC-3 — --since ISO date

**Command:**
```
sdlc changelog --since 2099-01-01
```

**Expected:**
- Exit 0
- Output: `No activity in the selected window.`

---

### TC-4 — --limit flag

**Setup:** 5 or more run records within last 7 days.

**Command:**
```
sdlc changelog --limit 2
```

**Expected:**
- Exit 0
- Exactly 2 lines of output (plus no trailing blank lines from extra events)

---

### TC-5 — --json output

**Setup:** At least one run record within last 7 days.

**Command:**
```
sdlc changelog --json
```

**Expected:**
- Exit 0
- Stdout is valid JSON
- Root object has keys: `since`, `limit`, `total`, `events`
- `events` is an array; each element has: `id`, `category`, `icon`, `label`, `run_type`, `status`, `started_at`

---

### TC-6 — --since last-merge with existing merge

**Setup:** At least one run record with `run_type == "merge"` in `.sdlc/.runs/`.

**Command:**
```
sdlc changelog --since last-merge
```

**Expected:**
- Exit 0
- Only events after the most recent merge appear (or empty state message if none followed the merge)

---

### TC-7 — --since last-merge with no merge runs

**Setup:** No run records with `run_type == "merge"`.

**Command:**
```
sdlc changelog --since last-merge
```

**Expected:**
- Exit 0
- Stderr contains a warning like "No merge found, defaulting to 7d"
- Falls back to 7-day window

---

### TC-8 — Empty .sdlc/.runs directory

**Setup:** `.sdlc/.runs/` is empty or does not exist.

**Command:**
```
sdlc changelog
```

**Expected:**
- Exit 0
- Output: `No activity in the selected window.`

---

### TC-9 — Invalid --since value

**Command:**
```
sdlc changelog --since "badvalue"
```

**Expected:**
- Exit non-zero
- Stderr contains "Invalid --since value"

---

### TC-10 — Run classification correctness

**Setup:** Run records with:
- `status = "failed"` → should show `⚠️`
- `run_type = "merge"` → should show `🚀`
- `run_type` contains "approve" → should show `✅`
- All other completed runs → should show `▶`

**Command:**
```
sdlc changelog --since 7d --json
```

**Expected:**
- `category` field in JSON matches classification rules
- `icon` field matches corresponding category icon

---

## Unit Test Coverage (Rust)

These are verified by `SDLC_NO_NPM=1 cargo test --all`:

| Test | Assertion |
|---|---|
| `test_classify_run_failed` | status="failed" → Category::RunFailed |
| `test_classify_merge` | run_type="merge" → Category::FeatureMerged |
| `test_classify_approval` | key contains "approve" → Category::Approval |
| `test_parse_since_relative` | "7d" → Relative(7 days); "1w" → Relative(7 days) |
| `test_parse_since_iso` | "2026-03-01" → Iso(2026-03-01 00:00 UTC) |
| `test_parse_since_invalid` | "bad" → Err |
| `test_relative_time_format` | 45s → "45 sec ago"; 90s → "1 min ago"; 3700s → "1 hr ago"; 90000s → "1 days ago" |

## Build Checks

- `SDLC_NO_NPM=1 cargo test --all` — passes
- `cargo clippy --all -- -D warnings` — passes
- `sdlc changelog --help` — exits 0, shows flag documentation
