mod cmd;
mod output;
mod root;
mod tools;

use clap::{Parser, Subcommand};
use cmd::{
    agent::AgentSubcommand, artifact::ArtifactSubcommand, auth::AuthSubcommand,
    backlog::BacklogSubcommand, comment::CommentSubcommand, config::ConfigSubcommand,
    escalate::EscalateSubcommand, feature::FeatureSubcommand, investigate::InvestigateSubcommand,
    knowledge::KnowledgeSubcommand, milestone::MilestoneSubcommand,
    orchestrate::OrchestrateSubcommand, platform::PlatformSubcommand, ponder::PonderSubcommand,
    project::ProjectSubcommand, query::QuerySubcommand, score::ScoreSubcommand,
    secrets::SecretsSubcommand, spike::SpikeSubcommand, task::TaskSubcommand,
    thread::ThreadSubcommand, tool::ToolCommand, ui::UiSubcommand,
};
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "sdlc",
    about = "Deterministic SDLC state machine — manage features, artifacts, tasks, and milestones",
    version = env!("SDLC_GIT_VERSION"),
    propagate_version = true
)]
struct Cli {
    /// Project root (default: auto-detect from .sdlc/ or .git/)
    #[arg(long, global = true, env = "SDLC_ROOT")]
    root: Option<PathBuf>,

    /// Output as JSON
    #[arg(long, global = true, short = 'j')]
    json: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize SDLC in the current project
    Init {
        /// Scaffold platform-specific scripts and config (e.g. masquerade)
        #[arg(long)]
        platform: Option<String>,
    },

    /// Show project state
    State,

    /// Classify the next action for a feature
    Next {
        /// Feature slug (omit to show all active features)
        #[arg(long = "for")]
        feature: Option<String>,
    },

    /// Show the single highest-priority actionable item (milestone order → feature order)
    Focus,

    /// Show up to 4 parallel work items across active milestones (same logic as dashboard)
    ParallelWork,

    /// Manage features
    Feature {
        #[command(subcommand)]
        subcommand: FeatureSubcommand,
    },

    /// Capture and manage out-of-scope concerns, ideas, and debt discovered during agent runs
    Backlog {
        #[command(subcommand)]
        subcommand: BacklogSubcommand,
    },

    /// Manage artifacts
    Artifact {
        #[command(subcommand)]
        subcommand: ArtifactSubcommand,
    },

    /// Manage tasks
    Task {
        #[command(subcommand)]
        subcommand: TaskSubcommand,
    },

    /// Add and list comments on features, tasks, and artifacts
    Comment {
        #[command(subcommand)]
        subcommand: CommentSubcommand,
    },

    /// Manage milestones
    Milestone {
        #[command(subcommand)]
        subcommand: MilestoneSubcommand,
    },

    /// Manage ponder space (pre-milestone ideation)
    Ponder {
        #[command(subcommand)]
        subcommand: PonderSubcommand,
    },

    /// Manage investigations (root-cause, evolve, guideline)
    Investigate {
        #[command(subcommand)]
        subcommand: InvestigateSubcommand,
    },

    /// Manage spike findings (list, show, promote)
    Spike {
        #[command(subcommand)]
        subcommand: SpikeSubcommand,
    },

    /// Manage the project knowledge base
    Knowledge {
        #[command(subcommand)]
        subcommand: KnowledgeSubcommand,
    },

    /// Run platform-specific commands (deploy, logs, dev, etc.)
    Platform {
        #[command(subcommand)]
        subcommand: PlatformSubcommand,
    },

    /// Project-level status, stats, and blockers
    Project {
        #[command(subcommand)]
        subcommand: ProjectSubcommand,
    },

    /// Query project state
    Query {
        #[command(subcommand)]
        subcommand: QuerySubcommand,
    },

    /// Validate the project configuration
    Config {
        #[command(subcommand)]
        subcommand: ConfigSubcommand,
    },

    /// Manage quality scores on features
    Score {
        #[command(subcommand)]
        subcommand: ScoreSubcommand,
    },

    /// Manage encrypted project secrets (AGE + SSH keys)
    Secrets {
        #[command(subcommand)]
        subcommand: SecretsSubcommand,
    },

    /// Manage named tunnel-access tokens (.sdlc/auth.yaml)
    Auth {
        #[command(subcommand)]
        subcommand: AuthSubcommand,
    },

    /// Escalate an action that requires human intervention
    Escalate {
        #[command(subcommand)]
        subcommand: EscalateSubcommand,
    },

    /// Manage feedback threads (contextual, append-only comment logs)
    Thread {
        #[command(subcommand)]
        subcommand: ThreadSubcommand,
    },

    /// Manage SDLC tool scripts (.sdlc/tools/)
    Tool {
        #[command(subcommand)]
        cmd: ToolCommand,
    },

    /// Run the tick-rate orchestrator daemon (or manage scheduled actions)
    Orchestrate {
        /// Seconds between ticks (default 60)
        #[arg(long, default_value_t = 60)]
        tick_rate: u64,
        /// Path to orchestrator DB (default: .sdlc/orchestrator.redb)
        #[arg(long)]
        db: Option<PathBuf>,
        #[command(subcommand)]
        subcommand: Option<OrchestrateSubcommand>,
    },

    /// Refresh agent scaffolding and stamp the current binary version
    Update,

    /// Show a digest of recent project activity (runs, merges, approvals)
    Changelog {
        /// Show events since: ISO date (2026-03-01), relative (7d, 1w), or last-merge
        #[arg(long, default_value = "7d")]
        since: String,
        /// Maximum events to show
        #[arg(long, default_value_t = 20)]
        limit: usize,
    },

    /// Commit changes to main with safe upstream merge
    Commit {
        /// Commit message (defaults to "wip")
        #[arg(long, short)]
        message: Option<String>,
    },

    /// Merge a feature (stub)
    Merge { slug: String },

    /// Archive a feature
    Archive { slug: String },

    /// Run as an MCP stdio server (used by claude-agent)
    Mcp,

    /// Drive a feature using an AI agent (programmatic equivalent of /sdlc-run)
    Agent {
        #[command(subcommand)]
        subcommand: AgentSubcommand,
    },

    /// Launch the web UI
    Ui {
        /// Port to listen on (0 = OS-assigned)
        #[arg(long, default_value = "0")]
        port: u16,

        /// Don't open browser automatically
        #[arg(long)]
        no_open: bool,

        /// Disable the public tunnel (tunnel starts automatically by default, requires orch-tunnel)
        #[arg(long)]
        no_tunnel: bool,

        /// Orchestrator tick interval in seconds (default 60)
        #[arg(long, default_value_t = 60)]
        tick_rate: u64,

        /// Start the orchestrator daemon and execute scheduled actions
        #[arg(long)]
        run_actions: bool,

        /// Enable debug-level logging (equivalent to RUST_LOG=debug)
        #[arg(long)]
        debug: bool,

        #[command(subcommand)]
        subcommand: Option<UiSubcommand>,
    },
}

fn main() {
    let cli = Cli::parse();

    let default_level = match &cli.command {
        Commands::Ui { debug: true, .. } => tracing::Level::DEBUG,
        Commands::Ui { .. } | Commands::Mcp => tracing::Level::INFO,
        _ => tracing::Level::WARN,
    };

    use tracing_subscriber::layer::SubscriberExt;
    use tracing_subscriber::util::SubscriberInitExt;

    let filter = tracing_subscriber::EnvFilter::from_default_env()
        .add_directive(default_level.into())
        .add_directive("redb=warn".parse().expect("valid directive"));
    let fmt_layer = tracing_subscriber::fmt::layer().with_target(false);

    if let Some(citadel_config) = sdlc_server::citadel::CitadelConfig::from_env() {
        let citadel_layer = sdlc_server::citadel::CitadelLayer::new(citadel_config);
        tracing_subscriber::registry()
            .with(filter)
            .with(fmt_layer)
            .with(citadel_layer)
            .init();
        // The flush background task is started later by start_citadel_flush()
        // once a tokio runtime is available (inside serve_on / serve_on_hub).
        // Events are buffered in the channel until then.
    } else {
        tracing_subscriber::registry()
            .with(filter)
            .with(fmt_layer)
            .init();
    }

    let root_path = cli.root.as_deref();
    // `sdlc init` always targets CWD — don't walk up to an ancestor .sdlc/
    let root = if matches!(cli.command, Commands::Init { .. }) {
        root_path
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")))
    } else {
        root::resolve_root(root_path)
    };

    let result = match cli.command {
        Commands::Init { platform } => cmd::init::run(&root, platform.as_deref()),
        Commands::State => cmd::state::run(&root, cli.json),
        Commands::Next { feature } => cmd::next::run(&root, feature.as_deref(), cli.json),
        Commands::Focus => cmd::focus::run(&root, cli.json),
        Commands::ParallelWork => cmd::parallel_work::run(&root, cli.json),
        Commands::Feature { subcommand } => cmd::feature::run(&root, subcommand, cli.json),
        Commands::Backlog { subcommand } => cmd::backlog::run(&root, subcommand, cli.json),
        Commands::Artifact { subcommand } => cmd::artifact::run(&root, subcommand, cli.json),
        Commands::Task { subcommand } => cmd::task::run(&root, subcommand, cli.json),
        Commands::Comment { subcommand } => cmd::comment::run(&root, subcommand, cli.json),
        Commands::Milestone { subcommand } => cmd::milestone::run(&root, subcommand, cli.json),
        Commands::Ponder { subcommand } => cmd::ponder::run(&root, subcommand, cli.json),
        Commands::Investigate { subcommand } => cmd::investigate::run(&root, subcommand, cli.json),
        Commands::Spike { subcommand } => cmd::spike::run(&root, subcommand, cli.json),
        Commands::Knowledge { subcommand } => cmd::knowledge::run(&root, subcommand, cli.json),
        Commands::Platform { subcommand } => cmd::platform::run(&root, subcommand, cli.json),
        Commands::Project { subcommand } => cmd::project::run(&root, subcommand, cli.json),
        Commands::Query { subcommand } => cmd::query::run(&root, subcommand, cli.json),
        Commands::Config { subcommand } => cmd::config::run(&root, subcommand, cli.json),
        Commands::Score { subcommand } => cmd::score::run(&root, subcommand, cli.json),
        Commands::Secrets { subcommand } => cmd::secrets::run(&root, subcommand, cli.json),
        Commands::Auth { subcommand } => cmd::auth::run(&root, subcommand, cli.json),
        Commands::Escalate { subcommand } => cmd::escalate::run(&root, subcommand, cli.json),
        Commands::Thread { subcommand } => cmd::thread::run(&root, subcommand, cli.json),
        Commands::Tool { cmd } => cmd::tool::run(cmd, &root),
        Commands::Orchestrate {
            tick_rate,
            db,
            subcommand,
        } => cmd::orchestrate::run(&root, subcommand, tick_rate, db),
        Commands::Update => cmd::update::run(&root),
        Commands::Changelog { since, limit } => cmd::changelog::run(&root, &since, limit, cli.json),
        Commands::Commit { message } => cmd::commit::run(&root, message.as_deref(), cli.json),
        Commands::Merge { slug } => cmd::merge::run(&root, &slug, cli.json),
        Commands::Archive { slug } => {
            cmd::feature::run(&root, FeatureSubcommand::Archive { slug }, cli.json)
        }
        Commands::Mcp => cmd::mcp::run(&root),
        Commands::Agent { subcommand } => cmd::agent::run(&root, subcommand, cli.json),
        Commands::Ui {
            port,
            no_open,
            no_tunnel,
            tick_rate,
            run_actions,
            debug: _,
            subcommand,
        } => cmd::ui::run(
            &root,
            subcommand,
            port,
            no_open,
            no_tunnel,
            tick_rate,
            run_actions,
        ),
    };

    if let Err(e) = result {
        // Print the full error chain (anyhow's alternate Display)
        eprintln!("error: {e:#}");
        std::process::exit(1);
    }
}
