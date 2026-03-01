use axum::{
    extract::{Path, State},
    Json,
};
use serde::Deserialize;
use tokio::task::spawn_blocking;

use crate::{
    error::AppError,
    state::{AppState, SseMessage},
};
use sdlc_core::advisory::{AdvisoryHistory, FindingStatus};

use super::runs::{sdlc_query_options, spawn_agent_run};

// ---------------------------------------------------------------------------
// GET /api/advisory
// ---------------------------------------------------------------------------

/// Return the full advisory history from `.sdlc/advisory.yaml`.
/// Returns an empty default if the file does not exist.
pub async fn get_advisory(State(app): State<AppState>) -> Result<Json<AdvisoryHistory>, AppError> {
    let root = app.root.clone();
    let history = spawn_blocking(move || AdvisoryHistory::load(&root))
        .await
        .map_err(|e| AppError(anyhow::anyhow!("task join error: {e}")))?
        .map_err(AppError::from)?;
    Ok(Json(history))
}

// ---------------------------------------------------------------------------
// PATCH /api/advisory/findings/:id
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
pub struct UpdateFindingBody {
    pub status: FindingStatus,
}

/// Update the status of a specific finding. Broadcasts an SSE Update so the
/// frontend refreshes any open advisory panel.
pub async fn update_finding(
    Path(id): Path<String>,
    State(app): State<AppState>,
    Json(body): Json<UpdateFindingBody>,
) -> Result<Json<sdlc_core::advisory::Finding>, AppError> {
    let root = app.root.clone();
    let id_clone = id.clone();
    let status = body.status;

    let result =
        spawn_blocking(move || AdvisoryHistory::update_finding_status(&root, &id_clone, status))
            .await
            .map_err(|e| AppError(anyhow::anyhow!("task join error: {e}")))??;

    match result {
        Some(finding) => {
            let _ = app.event_tx.send(SseMessage::Update);
            Ok(Json(finding))
        }
        None => Err(AppError::not_found(format!("Finding '{id}' not found"))),
    }
}

// ---------------------------------------------------------------------------
// POST /api/advisory/run
// ---------------------------------------------------------------------------

/// Start an advisory analysis agent run. The agent:
/// 1. Reads `.sdlc/advisory.yaml` for history context
/// 2. Orients to the maturity ladder stage
/// 3. Scans the codebase at appropriate depth
/// 4. Writes updated `.sdlc/advisory.yaml`
/// 5. Emits `advisory_run_completed` SSE when done
pub async fn start_advisory_run(
    State(app): State<AppState>,
) -> Result<Json<serde_json::Value>, AppError> {
    // Load history for context to include in the prompt
    let root_clone = app.root.clone();
    let history = spawn_blocking(move || AdvisoryHistory::load(&root_clone))
        .await
        .map_err(|e| AppError(anyhow::anyhow!("task join error: {e}")))?
        .unwrap_or_default();

    let history_context = if history.runs.is_empty() {
        "No previous advisory run. This is the first analysis.".to_string()
    } else {
        let last_run = history.runs.last().unwrap();
        let open_count = history
            .findings
            .iter()
            .filter(|f| matches!(f.status, sdlc_core::advisory::FindingStatus::Open))
            .count();
        format!(
            "Last advisory run: {}. Stage reached: {:?}. Open findings: {}. Total findings: {}.",
            last_run.run_at.format("%Y-%m-%d"),
            last_run.stage_reached,
            open_count,
            history.findings.len(),
        )
    };

    let prompt = format!(
        r#"You are an expert engineering advisor. Analyze this project's codebase and update the advisory history.

Context from previous runs: {history_context}

## Steps

1. Read `.sdlc/advisory.yaml` (if it exists) to understand the full history.
2. Read `VISION.md` and `ARCHITECTURE.md` if they exist.
3. Read feature and milestone state:
   ```bash
   sdlc feature list --json
   sdlc milestone list --json
   ```
4. Orient to the maturity ladder. Start where the history left off, or at Health for a fresh analysis.

   | Stage | What to check |
   |---|---|
   | **health** | Does it build? Tests pass? Dead code? Lint issues? |
   | **consistency** | One logging pattern? Config access DRY? Error shapes consistent? Naming uniform? |
   | **refactor** | Duplicated logic extracted? Files/functions over threshold? Missing abstractions? |
   | **structure** | Module boundaries respected? Common components DRY'd? No architectural drift? |
   | **roadmap** | Obvious feature gaps? Near-term user-facing improvements? Unfinished work? |
   | **advanced** | Strategic bets, ecosystem integrations, speculative improvements? |

   Decision logic:
   - If health findings are open and recent → stay at health, don't skip to roadmap
   - If history is old (>2 weeks) or the project has grown significantly → re-check current stage
   - If a stage is clean → move to the next one without dwelling
   - If a finding was dismissed/wont-fix → don't re-surface it

5. Scan the codebase at appropriate depth for the current stage. Read files; look for patterns that belong there.

6. Write updated `.sdlc/advisory.yaml`. Use this exact schema:

```yaml
runs:
  - run_at: "2026-02-28T12:00:00Z"    # ISO 8601
    file_count: 42                      # optional: approximate count
    stage_reached: health               # health | consistency | refactor | structure | roadmap | advanced
    summary: "One sentence describing what you found at this stage"
findings:
  - id: adv-a1b2c3                     # short identifier, e.g. adv- + 6 alphanum chars
    stage: health
    title: "Short title (5-8 words)"
    description: "Specific finding with file reference if applicable"
    status: open                        # open | acknowledged | resolved | dismissed
    created_at: "2026-02-28T12:00:00Z"
    resolved_at: null                   # omit or set when resolved
```

   - Append a new entry to `runs` (do not overwrite history).
   - For `findings`, merge with existing:
     - If a previous finding is no longer present in the code → mark it `resolved`
     - Add new findings with `open` status
     - Preserve findings marked `acknowledged` or `dismissed` — do not re-open them

7. Output a concise summary of what you found, grouped by stage, to help the developer decide what to ponder next."#
    );

    let opts = sdlc_query_options(app.root.clone(), 40);

    spawn_agent_run(
        "advisory".to_string(),
        prompt,
        opts,
        &app,
        "advisory",
        "Advisory analysis",
        Some(SseMessage::AdvisoryRunCompleted),
    )
    .await
}
