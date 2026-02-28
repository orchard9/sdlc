use anyhow::{Context, Result};
use claude_agent::{
    runner::{self, RunConfig},
    McpServerConfig, PermissionMode, QueryOptions,
};
use sdlc_core::{
    classifier::{Classification, Classifier, EvalContext},
    config::Config,
    feature::Feature,
    rules::default_rules,
    state::State,
    types::ActionType,
};
use std::collections::HashMap;
use std::path::Path;

// ---------------------------------------------------------------------------
// Subcommands
// ---------------------------------------------------------------------------

#[derive(clap::Subcommand)]
pub enum AgentSubcommand {
    /// Drive a feature forward autonomously to completion or the next HITL gate.
    ///
    /// Fires a headless Claude subprocess configured with the `sdlc mcp` tool
    /// server. Claude reads the current directive, executes the action, and
    /// loops until the feature reaches `done` or a human gate
    /// (`wait_for_approval`, `unblock_dependency`).
    Run {
        /// Feature slug to drive
        slug: String,

        /// Maximum agent turns (default: 200)
        #[arg(long, default_value = "200")]
        max_turns: u32,

        /// Model override (default: claude-sonnet-4-6)
        #[arg(long)]
        model: Option<String>,
    },
}

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

pub fn run(root: &Path, subcommand: AgentSubcommand, _json: bool) -> Result<()> {
    let AgentSubcommand::Run {
        slug,
        max_turns,
        model,
    } = subcommand;

    // Load state machine context
    let config = Config::load(root).context("failed to load config")?;
    let state = State::load(root).context("failed to load state")?;
    let feature =
        Feature::load(root, &slug).with_context(|| format!("feature '{slug}' not found"))?;

    let ctx = EvalContext {
        feature: &feature,
        state: &state,
        config: &config,
        root,
    };
    let classification = Classifier::new(default_rules()).classify(&ctx);

    // Short-circuit on terminal states — no need to spawn Claude
    match &classification.action {
        ActionType::Done => {
            println!("Feature '{slug}' is already done. Nothing to run.");
            return Ok(());
        }
        ActionType::WaitForApproval | ActionType::UnblockDependency => {
            println!(
                "Feature '{slug}' is at a human gate: {}",
                classification.action
            );
            println!("Resolve the gate, then re-run `sdlc agent run {slug}`.");
            return Ok(());
        }
        _ => {}
    }

    // Build MCP server config — points to the `sdlc mcp` subcommand of this
    // same binary. Claude will connect to it via JSON-RPC over stdio.
    let sdlc_bin = std::env::current_exe().unwrap_or_else(|_| std::path::PathBuf::from("sdlc"));
    tracing::info!(binary = %sdlc_bin.display(), cwd = %root.display(), "sdlc agent config");

    let mcp_server = McpServerConfig {
        name: "sdlc".into(),
        command: sdlc_bin.to_string_lossy().into_owned(),
        args: vec!["mcp".into()],
        env: HashMap::new(),
    };

    // All sdlc MCP tools are pre-approved; deny everything else silently.
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

    let opts = QueryOptions {
        model: model.or_else(|| Some("claude-sonnet-4-6".into())),
        max_turns: Some(max_turns),
        allowed_tools,
        permission_mode: PermissionMode::DontAsk,
        mcp_servers: vec![mcp_server],
        cwd: Some(root.to_path_buf()),
        ..Default::default()
    };

    let run_cfg = RunConfig {
        system_prompt: Some(build_system_prompt()),
        prompt: build_prompt(&slug, &classification),
        opts,
    };

    // Drive the agent — Claude handles the full directive loop internally via
    // MCP tool calls. We block until it completes (up to max_turns turns).
    tracing::info!(slug = %slug, max_turns, "spawning claude subprocess");
    let rt = tokio::runtime::Handle::try_current()
        .map(|_| None)
        .unwrap_or_else(|_| Some(tokio::runtime::Runtime::new().expect("tokio runtime")));

    let result = match rt {
        Some(rt) => {
            tracing::debug!("using new tokio runtime");
            rt.block_on(runner::run(run_cfg))
        }
        None => {
            // Already inside a runtime (e.g., integration test)
            tokio::task::block_in_place(|| {
                tokio::runtime::Handle::current().block_on(runner::run(run_cfg))
            })
        }
    }
    .context("agent run failed")?;

    println!("{}", result.result_text);
    println!("\n---");
    println!(
        "Turns: {}  Cost: ${:.4}",
        result.num_turns, result.total_cost_usd
    );

    if result.is_error {
        anyhow::bail!("agent run ended with an error result");
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Prompt builders
// ---------------------------------------------------------------------------

fn build_system_prompt() -> String {
    r#"You are an SDLC agent. You drive software features through a deterministic state machine.

You have access to these MCP tools (prefix: mcp__sdlc__):
- sdlc_get_directive   — Get the current action for a feature slug
- sdlc_write_artifact  — Write an artifact file and mark it as draft
- sdlc_approve_artifact — Approve a drafted artifact (advances phase)
- sdlc_reject_artifact  — Reject an artifact (sends it back for revision)
- sdlc_add_task         — Add a task to a feature
- sdlc_complete_task    — Mark a task complete
- sdlc_add_comment      — Add a comment or blocker to a feature

## Rules you must follow

1. After every action, call sdlc_get_directive to confirm state advanced.
2. Execute exactly one action per loop iteration — never batch artifact writes.
3. Never call sdlc_approve_artifact without first calling sdlc_write_artifact.
4. Stop and report clearly when action is `done`, `wait_for_approval`, or `unblock_dependency`.
5. Do not guess or invent actions — the directive is always authoritative.

## Artifact types
spec · design · tasks · qa_plan · review · audit · qa_results

## Phase flow
DRAFT → SPECIFIED → PLANNED → READY → IMPLEMENTATION → REVIEW → AUDIT → QA → MERGE → done
"#
    .to_string()
}

fn build_prompt(slug: &str, classification: &Classification) -> String {
    let directive_json = serde_json::to_string_pretty(classification)
        .unwrap_or_else(|_| format!("{classification:?}"));

    format!(
        "Drive feature '{slug}' forward using the sdlc state machine tools.\n\n\
         Current directive:\n{directive_json}\n\n\
         Execute the action, verify state advanced with sdlc_get_directive, then loop \
         until done or a human gate (wait_for_approval / unblock_dependency)."
    )
}
