## Summary

QA covers three verification areas: (1) visual inspection of the teaser row in the `ArtifactViewer` card for all artifact states, (2) unit-level logic verification for `extractTeaser` and `formatRelativeTime` via manual test cases, and (3) content inspection of the updated `sdlc-run` and `sdlc-next` command files to confirm the `## Summary` convention text is present and correctly placed.

## Scope

- `frontend/src/components/features/ArtifactViewer.tsx` — teaser row rendering
- `crates/sdlc-cli/src/cmd/init/commands/sdlc_run.rs` — convention callout in run command
- `crates/sdlc-cli/src/cmd/init/commands/sdlc_next.rs` — convention instruction in next command
- Build smoke test: `SDLC_NO_NPM=1 cargo build --all` must succeed; `cargo clippy --all -- -D warnings` must be clean

## Test Cases

### TC1: Teaser row — artifact with ## Summary section
**Expected:** Card header shows teaser row with Summary section content.

### TC2: Teaser row — artifact without ## Summary, has body paragraph
**Expected:** Teaser row shows first body paragraph.

### TC3: Teaser row — long teaser truncation
**Expected:** Teaser is truncated to 117 characters + `…`.

### TC4: Teaser row — missing artifact
**Expected:** No teaser row rendered. Card shows "Not created yet" as before.

### TC5: Timestamp — artifact with approved_at
**Expected:** Teaser row shows `Clock` icon + relative time text.

### TC6: Timestamp — artifact with no approved_at
**Expected:** Teaser row shows teaser text only, no clock icon, no separator `·`.

### TC7: Teaser row absent when extraction yields empty string
**Expected:** No teaser row rendered.

### TC8: extractTeaser unit logic — H1 skip
**Input:** `"# Title\nFirst paragraph."`, **Expected:** `"First paragraph."`

### TC9: extractTeaser unit logic — Summary preference
**Input:** `"# Title\nPreamble.\n## Summary\nSummary text.\n## Other\nOther."`, **Expected:** `"Summary text."`

### TC10: formatRelativeTime — seconds
**Input:** ISO string 30 seconds ago, **Expected:** `"30s ago"`

### TC11: formatRelativeTime — minutes
**Input:** ISO string 90 seconds ago, **Expected:** `"1m ago"`

### TC12: formatRelativeTime — hours
**Input:** ISO string 2.5 hours ago, **Expected:** `"2h ago"`

### TC13: formatRelativeTime — days
**Input:** ISO string 3 days ago, **Expected:** `"3d ago"`

### TC14: formatRelativeTime — over a month
**Input:** ISO string 45 days ago, **Expected:** `"over a month ago"`

### TC15: sdlc-next command contains ## Summary instruction
**Check:** Confirm `SDLC_NEXT_COMMAND` contains "## Summary" and "2–4 sentences". Confirm `SDLC_NEXT_PLAYBOOK` contains "populates the UI teaser card".

### TC16: sdlc-run command contains ## Summary convention callout
**Check:** Confirm `SDLC_RUN_COMMAND` contains "## Summary convention". Confirm `SDLC_RUN_PLAYBOOK` contains the new step with "## Summary".

### TC17: Build and clippy clean
**Command:** `SDLC_NO_NPM=1 cargo build --all && cargo clippy --all -- -D warnings`
**Expected:** Zero errors, zero warnings.

### TC18: Fullscreen modal unaffected
**Expected:** Fullscreen modal renders only `MarkdownContent` — no teaser row inside the modal.
