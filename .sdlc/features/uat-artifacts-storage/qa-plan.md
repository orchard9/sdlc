# QA Plan: UAT Artifact Storage

## Scope

Three changes to verify:
1. `UatRun.screenshot_paths` field — model correctness and backward compat
2. `GET /api/milestones/:slug/uat-runs/:run_id/artifacts/:filename` — happy path, 404, 400 (path traversal)
3. `start_milestone_uat` prompt — contains required screenshot instructions

---

## TC-1: UatRun backward compatibility

**Verify:** YAML without `screenshot_paths` deserializes to an empty vec.

```rust
let yaml = "id: test\nmilestone_slug: v1\nstarted_at: 2026-01-01T00:00:00Z\nverdict: pass\ntests_total: 1\ntests_passed: 1\ntests_failed: 0\nsummary_path: foo.md\n";
let run: UatRun = serde_yaml::from_str(yaml).unwrap();
assert!(run.screenshot_paths.is_empty());
```

**Pass criteria:** no panic, `screenshot_paths` is empty.

---

## TC-2: UatRun round-trip with screenshot_paths

**Verify:** a `UatRun` with `screenshot_paths` serializes and deserializes correctly.

```rust
let run = UatRun {
    screenshot_paths: vec![
        ".sdlc/milestones/v1/uat-runs/20260101-abc/01-login.png".to_string(),
    ],
    // ... other fields
};
let yaml = serde_yaml::to_string(&run).unwrap();
let loaded: UatRun = serde_yaml::from_str(&yaml).unwrap();
assert_eq!(loaded.screenshot_paths, run.screenshot_paths);
```

**Pass criteria:** paths round-trip exactly.

---

## TC-3: Artifact route — happy path PNG

**Verify:** `GET /api/milestones/v1/uat-runs/run1/artifacts/screenshot.png` returns 200 with `Content-Type: image/png` and the file bytes.

Setup: write a valid PNG file to the run directory.

**Pass criteria:** status 200, correct `Content-Type`, body matches file bytes.

---

## TC-4: Artifact route — 404 for missing file

**Verify:** requesting a filename that does not exist returns 404.

**Pass criteria:** status 404.

---

## TC-5: Artifact route — 400 for path traversal

**Verify:** filenames with `..` or `/` return 400 Bad Request.

Test inputs:
- `../../../etc/passwd`
- `foo/../bar.png`
- `subdir/file.png`

**Pass criteria:** all return 400.

---

## TC-6: MIME type detection

**Verify:** the `mime_for_filename` helper returns the correct MIME type for each extension.

| Input | Expected |
|---|---|
| `foo.png` | `image/png` |
| `foo.jpg` | `image/jpeg` |
| `foo.jpeg` | `image/jpeg` |
| `foo.webm` | `video/webm` |
| `foo.mp4` | `video/mp4` |
| `foo.gif` | `image/gif` |
| `foo.bin` | `application/octet-stream` |
| `no_extension` | `application/octet-stream` |

**Pass criteria:** all match.

---

## TC-7: Agent prompt contains screenshot instructions

**Verify:** the `prompt` string built in `start_milestone_uat` contains:
- `"run_id"` (instructs agent to generate a run ID)
- `"screenshot"` (instructs agent to take screenshots)
- `"screenshot_paths"` (instructs agent to write the field)

This is a code-level assertion check on the string content.

**Pass criteria:** all three substrings present.

---

## TC-8: Build and lint

```bash
SDLC_NO_NPM=1 cargo test --all
cargo clippy --all -- -D warnings
```

**Pass criteria:** zero test failures, zero clippy warnings.

---

## Execution

All TC-1 through TC-7 are implemented as Rust unit tests in the relevant crates. TC-8 is run as a build verification step after all code changes are in place.
