# Feature Breakdown: Playwright Artifacts

## Prerequisite Discovery

The UAT history panel currently shows 'No UAT runs yet' even though 3 run dirs exist in v08. Root cause: agent writes `summary.md` but Rust reads `run.yaml`. The first fix is making the agent write structured `run.yaml`.

## Feature 1: uat-artifacts-storage

**Scope:** Rust backend + agent prompt fixes

### Tasks
- T1: Add `screenshots: Vec<String>` and `video_path: Option<String>` to `UatRun` struct in `sdlc-core/src/milestone.rs`
- T2: Add `ux_flow: bool` field to milestone manifest struct (default false)
- T3: Add `GET /api/milestones/{slug}/uat-runs/{run_id}/artifacts/{filename}` — binary file serving from run dir, path-traversal safe, Content-Type by extension
- T4: Add `sdlc milestone uat-run save <slug> --json <payload>` CLI command so agent can write structured `run.yaml`
- T5: Fix UAT agent prompt in `runs.rs`:
  - Specify screenshot save path: `<run_dir>/screenshots/<step>-<label>.png`
  - Instruct agent to call `sdlc milestone uat-run save` at the end to write `run.yaml`
  - Conditionally add `--video=on --output-dir <run_dir>` to Playwright MCP args if milestone has `ux_flow: true`

### Directory shape after T1-T5
```
.sdlc/milestones/<slug>/uat-runs/<run-id>/
  run.yaml                         # UatRun struct
  summary.md                       # agent-written human summary
  screenshots/
    01-before-login.png
    02-dashboard-loaded.png
  video.webm                       # only if ux_flow: true
```

## Feature 2: uat-artifacts-ui

**Scope:** Frontend only — depends on Feature 1

### Tasks
- T1: Extend TypeScript `UatRun` type: add `screenshots: string[]` and `video_path: string | null`
- T2: Update `UatHistoryPanel`: show screenshot filmstrip (up to 3 thumbnails per run row); show `▶ video` link if `video_path` set. Click screenshot → open full size in new tab via artifacts endpoint.
- T3: Dashboard milestone card: show hero screenshot (first screenshot of latest run) as subtle inset on the milestone card if available
- T4: (Optional/future) Link to Playwright HTML report via `playwright_report_path` if present

## Key Decisions

- **`ux_flow` in milestone manifest.yaml** — not inferred, explicitly declared
- **Binary serving**: new dedicated route, not static file middleware (keeps .sdlc access controlled)
- **Screenshot naming**: `<step-number>-<label>.png` convention in agent prompt
- **Video format**: webm (from `@playwright/mcp --video=on`)
- **Dashboard surface**: verdict + hero screenshot thumbnail only. Gallery on milestone detail page.

## Open Questions

- Does `@playwright/mcp --video=on` actually work? Need a spike to confirm the flag and output format.
- Should we backfill existing run dirs (add `run.yaml`) or leave them as-is? Likely leave as-is — they'll appear as zero-count in the UI but that's better than a migration script.
