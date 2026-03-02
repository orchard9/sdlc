---
session: 1
timestamp: 2026-03-02T09:45:00Z
orientation:
  current: "Core storage shape decided. Video via MCP --video flag is viable. Binary serving route needed. Dashboard surface: screenshot thumbnails inline in UatHistoryPanel, video link on verdict row."
  next: "Create features: uat-artifacts-storage (Rust data model + binary serving), uat-artifacts-ui (UatHistoryPanel thumbnails + video link, dashboard latest-run card with screenshot)"
  commit: "Two features scoped and ready to spec. Commit signal met."
---

**jordan · Owner**
playwright artifacts

when we store uat results/playwright artifacts they should be in .sdlc and we should surface them as artifacts and in the new dashboard

we should also record videos if its a ux flow as well as the screenshots

---

**Facilitator**

Good seed. Let me pull in the people who can stress-test this. Nadia on artifact surface, Priya on storage, Tobias on scope, Ben on what developers actually need to see.

Before anyone speaks — let me frame the real problem by looking at what exists. UAT runs today:

- `.sdlc/milestones/<slug>/uat-runs/<run-id>/summary.md` — human-written markdown summary by the agent
- `UatRun` struct has `playwright_report_path: Option<String>` — set to `null` in every run that's ever happened
- The UAT agent takes screenshots via `mcp__playwright__browser_take_screenshot` but never saves them anywhere persistent — they go to a temp path or just get consumed by the agent and forgotten
- `UatHistoryPanel` shows verdict badges and pass/fail counts. No visual evidence.

The real problem: **UAT runs happen, things pass or fail, but there's zero visual evidence.** When a run fails, you get a text summary. When it passes, you have no proof. Screenshots and videos are the proof layer.

---

**Tobias Krenn · Skeptical Engineering Lead**

Before anyone designs a media gallery — what's the actual pain? Jordan's seeing failing runs in `v08` with 6/13 passes. The failures are completely clear in text: "redb lock held by daemon." Screenshots would add zero information there.

So when *does* visual evidence matter? It matters when:
1. The agent claims a UX flow passed but you suspect it navigated to the wrong page
2. You want to prove to a stakeholder that the UI actually renders correctly
3. You're debugging a flaky test and need to see what state the browser was in

That's specific. Not "always record video." The scope should be: **screenshots on scenario completion, video only when `acceptance_test.md` declares `kind: ux_flow`**.

? **Open:** Should `kind: ux_flow` be a field in `acceptance_test.md` frontmatter, or should the UAT agent infer it from whether browser navigation happens?

---

**Priya Nair · Distributed Systems**

Let me ground the storage question before the UI conversation runs away. Here's the current layout:

```
.sdlc/milestones/<slug>/uat-runs/<run-id>/
  summary.md          # agent-written markdown (not a run.yaml — mismatch with Rust code!)
```

The Rust `save_uat_run()` writes to `run.yaml` but existing dirs only have `summary.md`. That's a format inconsistency the code currently papers over by only reading `run.yaml` and ignoring dirs without it.

For artifacts the shape should be:
```
.sdlc/milestones/<slug>/uat-runs/<run-id>/
  run.yaml                      # UatRun struct (verdict, counts, paths)
  summary.md                    # human-readable agent summary
  screenshots/
    01-before-login.png
    02-after-login.png
    03-dashboard.png
  video.webm                    # optional, only for ux_flow milestones
```

**Key constraint:** The server currently serves no binary files from `.sdlc/`. To expose screenshots and video to the frontend, we need a new route:

```
GET /api/milestones/{slug}/uat-runs/{run_id}/artifacts/{filename}
```

This is just `tokio::fs::read()` + `Content-Type` inference by extension. Straightforward. The path is validated to stay within the run directory (no path traversal).

⚑ **Decided:** Screenshots stored at `screenshots/*.png` inside the run dir. Video at `video.webm`. New `GET /api/milestones/{slug}/uat-runs/{run_id}/artifacts/{filename}` serves binary files with correct Content-Type.

---

**Nadia Osei · Rich Document Experience**

The dashboard ask is the interesting part. "Surface them in the new dashboard" — what does that actually mean?

Looking at `UatHistoryPanel` — it's a list of verdict badges with pass/fail counts. That's the history view on the milestone detail page. The "new dashboard" (`dashboard-rethink`) is a project digest. Those are two different surfaces.

**On the milestone detail / UatHistoryPanel:**
Inline screenshot thumbnails would be powerful here. Each run row gets a horizontal filmstrip of 2-3 key screenshots. Click to expand. That's the evidence you want: "here's what the agent saw when this passed." For video, a play button on the row if `video_path` is set.

**On the dashboard (project digest):**
The dashboard doesn't need screenshot galleries. It needs signal: "last UAT run for v08: FAILED — 6/13". A thumbnail of the final screenshot could appear as a subtle preview on the milestone card. But the full gallery belongs on the milestone page.

⚑ **Decided:** Dashboard shows latest UAT verdict + date + pass/fail ratio (already in the cards, this is the SSE-driven update). Optional: single "hero" screenshot thumbnail on the milestone card if `screenshots` array is non-empty. Full filmstrip lives in UatHistoryPanel on the milestone detail page.

---

**Ben Hartley · Developer Productivity UX**

Let me push on the video question specifically. "Record videos if it's a UX flow" — how does the Playwright MCP do this?

I looked at `@playwright/mcp` — the server can be launched with `--video=on` flag which enables automatic video recording. Videos are saved alongside screenshots. This means the `start_milestone_uat` in `runs.rs` just needs:

```rust
opts.mcp_servers.push(McpServerConfig {
    name: "playwright".into(),
    command: "npx".into(),
    args: vec!["@playwright/mcp@latest".into(), "--video=on".into(), "--output-dir".into(), video_dir.to_str().unwrap().into()],
    ...
});
```

But that records video for *all* UAT runs, even API-only ones where the browser is never opened. That's wasteful.

Alternative: the UAT agent prompt tells it to call `browser_run_code` with Playwright's `page.video()` API to save a video file. But that's more complex and relies on the agent doing the right thing.

Simplest: conditional. Read the `acceptance_test.md` frontmatter for `ux_flow: true`. If present, add `--video=on` to the Playwright MCP args. If not, no video.

? **Open:** How does `start_milestone_uat` know if the milestone is a UX flow? Two options:
1. Parse `acceptance_test.md` frontmatter in Rust before spawning (adds file-read to the HTTP handler path)
2. Declare `ux_flow` in `milestone manifest.yaml` (cleaner, milestone-level metadata)

My vote: milestone manifest has `ux_flow: bool`. Default false. UAT route reads it before spawning. One field, one read.

---

**Tobias Krenn**

I want to audit the UAT agent prompt before we add more complexity. Current prompt:

> "Write signed checklist results to `.sdlc/milestones/{slug}/uat_results.md`."

It says nothing about:
- Where to save screenshots (the `filename` param in `browser_take_screenshot`)
- What naming convention to use
- That screenshots should go in `screenshots/` subdirectory of the run dir
- That `run.yaml` should be created via `sdlc milestone uat-run save`

The agent is flying blind on artifact storage. Even before we add video, we need to fix the agent instructions. The prompt needs to tell the agent:
1. Resolve the run dir: `.sdlc/milestones/{slug}/uat-runs/{run_id}/`
2. Save screenshots to `screenshots/<step>.png` relative to that dir
3. Write `run.yaml` via `sdlc milestone uat-run save` CLI command (or REST PATCH)
4. Write `summary.md` in the same dir

⚑ **Decided:** Fix the UAT agent prompt to specify screenshot save paths. Add a CLI command `sdlc milestone uat-run save` (or extend existing) that writes `run.yaml` with the full `UatRun` struct including `screenshots` array and optional `video_path`.

---

**Priya Nair**

On the `UatRun` data model extension:

```rust
pub struct UatRun {
    // existing fields...
    pub id: String,
    pub milestone_slug: String,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub verdict: UatVerdict,
    pub tests_total: u32,
    pub tests_passed: u32,
    pub tests_failed: u32,
    pub playwright_report_path: Option<String>,  // keep for HTML report link
    pub tasks_created: Vec<String>,
    pub summary_path: String,
    // NEW:
    pub screenshots: Vec<String>,   // relative paths within run dir, e.g. ["screenshots/01-login.png"]
    pub video_path: Option<String>, // relative path, e.g. "video.webm"
}
```

The existing `playwright_report_path` field was presumably intended for Playwright's HTML report (from `npx playwright show-report`). We should keep it. It's a different artifact — the full HTML test report — versus individual screenshots.

⚑ **Decided:** Add `screenshots: Vec<String>` and `video_path: Option<String>` to `UatRun`. Keep `playwright_report_path` for the Playwright HTML report.

---

**Nadia Osei**

Let me sketch the `UatHistoryPanel` redesign:

```
┌──────────────────────────────────────────────────────────────────┐
│ [FAILED] Mar 2  6/13 passed  · 1 task created                   │
│ ┌─────────────┐ ┌─────────────┐ ┌─────────────┐                 │
│ │ [screenshot]│ │ [screenshot]│ │ [screenshot]│  ▶ video         │
│ └─────────────┘ └─────────────┘ └─────────────┘                 │
└──────────────────────────────────────────────────────────────────┘
┌──────────────────────────────────────────────────────────────────┐
│ [PASS] Mar 2  13/13 passed                                       │
│ ┌─────────────┐ ┌─────────────┐                                  │
│ │ [screenshot]│ │ [screenshot]│                                   │
│ └─────────────┘ └─────────────┘                                  │
└──────────────────────────────────────────────────────────────────┘
```

Screenshots are served as `<img src="/api/milestones/{slug}/uat-runs/{run_id}/artifacts/screenshots/01.png">`. Thumbnails, click to expand in a lightbox (or just link to full size — no need for a custom lightbox component).

For the dashboard: milestone card shows the verdict badge and optionally a single hero screenshot (first screenshot of the latest run) as a subtle background or inset image. Keeps the digest feeling, not a media browser.

---

**Ben Hartley**

Last thing I want to flag: **the existing `summary.md` vs `run.yaml` inconsistency.** Looking at the real files:

```
.sdlc/milestones/v08-orchestrator-webhooks/uat-runs/2026-03-02-001/summary.md
```

The Rust code reads `run.yaml` (via `uat_run_manifest`), not `summary.md`. So `list_uat_runs()` currently returns zero runs from the API, because none of these dirs have a `run.yaml`. The `UatHistoryPanel` always shows "No UAT runs yet" even though there are 3 run dirs.

This means **the migration fix (writing `run.yaml`) must happen before the UI work has anything to display.** The agent needs to be told to call a CLI command or REST endpoint to write the structured `run.yaml` as part of finalizing each run.

⚑ **Decided:** Before building the artifact UI, fix the agent to write `run.yaml` (structured `UatRun`) at the end of each UAT run. This is the prerequisite — without it `UatHistoryPanel` shows nothing.

---

**Facilitator — Synthesis**

Here's where we landed:

### What we're building

**Feature 1: `uat-artifacts-storage`** (Rust, 3-4 tasks)
- Add `screenshots: Vec<String>` and `video_path: Option<String>` to `UatRun` struct
- Add `ux_flow: bool` to milestone manifest (default false)
- Add `GET /api/milestones/{slug}/uat-runs/{run_id}/artifacts/{filename}` — binary file serving from run dir (Content-Type by extension, path-traversal safe)
- Add `sdlc milestone uat-run save <slug> <json>` CLI command so the agent can write `run.yaml` with full UatRun struct
- Fix the UAT agent prompt in `runs.rs` to: specify screenshot save paths, create run dir, write run.yaml via CLI, save video if `ux_flow: true`
- Add `--video=on --output-dir <run_dir>` to Playwright MCP args when milestone has `ux_flow: true`

**Feature 2: `uat-artifacts-ui`** (Frontend, 3-4 tasks)
- Extend `UatRun` TypeScript type with `screenshots` and `video_path`
- Update `UatHistoryPanel`: show screenshot filmstrip (2-3 thumbnails) per run row; show `▶ video` link if `video_path` set
- Dashboard milestone card: show hero screenshot (first of latest run) if available
- Add lightbox or fullscreen view for screenshot (simple: open in new tab via link to the artifacts endpoint)

### Open questions resolved
- ⚑ `ux_flow` declared in milestone `manifest.yaml`, not inferred. Agent prompt updated to say "if milestone has `ux_flow: true`, video will be recorded automatically."
- ⚑ Screenshot naming: agent uses step number prefix + descriptive label, e.g. `screenshots/01-before-login.png`. Convention specified in UAT agent prompt.
- ⚑ Dashboard surface: verdict badge + hero screenshot thumbnail (not a full gallery). Gallery is on milestone detail page.

### Still open
? **Open:** Should we expose a Playwright HTML report? The `playwright_report_path` field exists. This could be a future feature — for now, leave it null.
? **Open:** Video codec. `@playwright/mcp --video=on` produces what format? Likely webm. Need to confirm `<video>` element in the browser can play it directly.
