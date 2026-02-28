use super::SdlcTool;
use std::path::Path;

pub struct RunWaveTool;

impl SdlcTool for RunWaveTool {
    fn name(&self) -> &str {
        "sdlc_run_wave"
    }

    fn description(&self) -> &str {
        "Execute Wave 1 from the prepare plan — spawns parallel agents for each feature"
    }

    fn schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "milestone": {
                    "type": "string",
                    "description": "Milestone slug (auto-detects if omitted)"
                },
                "max_parallel": {
                    "type": "integer",
                    "description": "Maximum parallel agents (default: 3)"
                }
            },
            "required": []
        })
    }

    fn call(&self, args: serde_json::Value, root: &Path) -> Result<serde_json::Value, String> {
        let milestone = args["milestone"].as_str();
        let max_parallel = args["max_parallel"].as_u64().unwrap_or(3) as usize;

        // 1. Get the wave plan
        let prepare_result =
            sdlc_core::prepare::prepare(root, milestone).map_err(|e| e.to_string())?;

        if prepare_result.waves.is_empty() {
            return Ok(serde_json::json!({
                "status": "nothing_to_run",
                "project_phase": prepare_result.project_phase,
                "message": "No actionable features in the current wave plan."
            }));
        }

        let wave1 = &prepare_result.waves[0];

        // 2. Separate items that need worktrees (skip them) from those that don't
        let mut runnable = Vec::new();
        let mut skipped_needs_worktree = Vec::new();

        for item in &wave1.items {
            if item.needs_worktree {
                skipped_needs_worktree.push(serde_json::json!({
                    "slug": item.slug,
                    "phase": item.phase,
                    "action": item.action,
                    "reason": "Needs worktree — run manually with /sdlc-run in a worktree"
                }));
            } else {
                runnable.push(item.clone());
            }
        }

        if runnable.is_empty() {
            return Ok(serde_json::json!({
                "status": "all_need_worktrees",
                "wave": wave1.number,
                "skipped_needs_worktree": skipped_needs_worktree,
                "message": "All Wave 1 features require worktrees. Run them manually."
            }));
        }

        // 3. Build RunConfigs for each runnable feature
        let sdlc_bin = std::env::current_exe().unwrap_or_else(|_| std::path::PathBuf::from("sdlc"));

        let config = sdlc_core::config::Config::load(root).map_err(|e| e.to_string())?;
        let state = sdlc_core::state::State::load(root).map_err(|e| e.to_string())?;
        let classifier = sdlc_core::classifier::Classifier::new(sdlc_core::rules::default_rules());

        let configs: Vec<(String, claude_agent::runner::RunConfig)> = runnable
            .iter()
            .filter_map(|item| {
                let feature = sdlc_core::feature::Feature::load(root, &item.slug).ok()?;

                let ctx = sdlc_core::classifier::EvalContext {
                    feature: &feature,
                    state: &state,
                    config: &config,
                    root,
                };
                let classification = classifier.classify(&ctx);

                // Skip terminal states
                if matches!(
                    classification.action,
                    sdlc_core::types::ActionType::Done
                        | sdlc_core::types::ActionType::WaitForApproval
                        | sdlc_core::types::ActionType::UnblockDependency
                ) {
                    return None;
                }

                let mcp_server = claude_agent::McpServerConfig {
                    name: "sdlc".into(),
                    command: sdlc_bin.to_string_lossy().into_owned(),
                    args: vec!["mcp".into()],
                    env: std::collections::HashMap::new(),
                };

                let allowed_tools = vec![
                    "mcp__sdlc__sdlc_get_directive".into(),
                    "mcp__sdlc__sdlc_write_artifact".into(),
                    "mcp__sdlc__sdlc_approve_artifact".into(),
                    "mcp__sdlc__sdlc_reject_artifact".into(),
                    "mcp__sdlc__sdlc_add_task".into(),
                    "mcp__sdlc__sdlc_complete_task".into(),
                    "mcp__sdlc__sdlc_add_comment".into(),
                    "mcp__sdlc__sdlc_project_phase".into(),
                    "mcp__sdlc__sdlc_prepare".into(),
                ];

                let opts = claude_agent::QueryOptions {
                    model: Some("claude-sonnet-4-6".into()),
                    max_turns: Some(200),
                    allowed_tools,
                    permission_mode: claude_agent::PermissionMode::DontAsk,
                    mcp_servers: vec![mcp_server],
                    cwd: Some(root.to_path_buf()),
                    ..Default::default()
                };

                let directive_json = serde_json::to_string_pretty(&classification)
                    .unwrap_or_else(|_| format!("{classification:?}"));

                let run_cfg = claude_agent::runner::RunConfig {
                    system_prompt: Some(build_system_prompt()),
                    prompt: format!(
                        "Drive feature '{}' forward using the sdlc state machine tools.\n\n\
                         Current directive:\n{directive_json}\n\n\
                         Execute the action, verify state advanced with sdlc_get_directive, then loop \
                         until done or a human gate (wait_for_approval / unblock_dependency).",
                        item.slug
                    ),
                    opts,
                };

                Some((item.slug.clone(), run_cfg))
            })
            .collect();

        if configs.is_empty() {
            return Ok(serde_json::json!({
                "status": "nothing_runnable",
                "wave": wave1.number,
                "skipped_needs_worktree": skipped_needs_worktree,
                "message": "No features in Wave 1 are in a runnable state."
            }));
        }

        // 4. Spawn parallel runs
        let root_owned = root.to_path_buf();
        let rt = match tokio::runtime::Handle::try_current() {
            Ok(_) => None,
            Err(_) => Some(
                tokio::runtime::Runtime::new()
                    .map_err(|e| format!("failed to create tokio runtime: {e}"))?,
            ),
        };

        let results = match &rt {
            Some(rt) => rt.block_on(run_wave_async(configs, max_parallel)),
            None => tokio::task::block_in_place(|| {
                tokio::runtime::Handle::current().block_on(run_wave_async(configs, max_parallel))
            }),
        };

        // 5. Re-run prepare to get updated wave plan
        let updated_prepare = sdlc_core::prepare::prepare(&root_owned, milestone).ok();

        // 6. Collect results
        let feature_results: Vec<serde_json::Value> = results
            .into_iter()
            .map(|(slug, outcome)| match outcome {
                Ok(run_result) => serde_json::json!({
                    "slug": slug,
                    "status": if run_result.is_error { "error" } else { "completed" },
                    "result_text": run_result.result_text,
                    "turns": run_result.num_turns,
                    "cost_usd": run_result.total_cost_usd,
                }),
                Err(err) => serde_json::json!({
                    "slug": slug,
                    "status": "failed",
                    "error": err,
                }),
            })
            .collect();

        let completed = feature_results
            .iter()
            .filter(|r| r["status"] == "completed")
            .count();
        let failed = feature_results.len() - completed;

        Ok(serde_json::json!({
            "status": "wave_complete",
            "wave": wave1.number,
            "features_run": feature_results.len(),
            "completed": completed,
            "failed": failed,
            "results": feature_results,
            "skipped_needs_worktree": skipped_needs_worktree,
            "updated_wave_plan": updated_prepare.and_then(|p| serde_json::to_value(p).ok()),
        }))
    }
}

async fn run_wave_async(
    configs: Vec<(String, claude_agent::runner::RunConfig)>,
    max_parallel: usize,
) -> Vec<(String, Result<claude_agent::runner::RunResult, String>)> {
    let semaphore = std::sync::Arc::new(tokio::sync::Semaphore::new(max_parallel));
    let mut handles = Vec::new();

    for (slug, config) in configs {
        let sem = semaphore.clone();
        let handle = tokio::spawn(async move {
            let _permit = match sem.acquire().await {
                Ok(p) => p,
                Err(_) => return (slug, Err("semaphore closed".to_string())),
            };
            let result = claude_agent::runner::run(config)
                .await
                .map_err(|e| e.to_string());
            (slug, result)
        });
        handles.push(handle);
    }

    let mut results = Vec::new();
    for handle in handles {
        match handle.await {
            Ok(r) => results.push(r),
            Err(e) => results.push(("unknown".into(), Err(format!("task join error: {e}")))),
        }
    }
    results
}

fn build_system_prompt() -> String {
    r#"You are an SDLC agent. You drive software features through a deterministic state machine.

You have access to these MCP tools (prefix: mcp__sdlc__):
- sdlc_get_directive    — Get the current action for a feature slug
- sdlc_write_artifact   — Write an artifact file and mark it as draft
- sdlc_approve_artifact — Approve a drafted artifact (advances phase)
- sdlc_reject_artifact  — Reject an artifact (sends it back for revision)
- sdlc_add_task         — Add a task to a feature
- sdlc_complete_task    — Mark a task complete
- sdlc_add_comment      — Add a comment or blocker to a feature
- sdlc_project_phase    — Get the current project lifecycle phase
- sdlc_prepare          — Survey the milestone for wave plan and gaps

## Rules you must follow

1. After every action, call sdlc_get_directive to confirm state advanced.
2. Execute exactly one action per loop iteration — never batch artifact writes.
3. Never call sdlc_approve_artifact without first calling sdlc_write_artifact.
4. Stop and report clearly when action is `done`, `wait_for_approval`, or `unblock_dependency`.
5. Do not guess or invent actions — the directive is always authoritative.

## Phase flow
DRAFT → SPECIFIED → PLANNED → READY → IMPLEMENTATION → REVIEW → AUDIT → QA → MERGE → done
"#
    .to_string()
}
