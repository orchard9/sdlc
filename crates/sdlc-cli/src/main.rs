mod cmd;
mod output;
mod root;
mod tools;

use clap::{Parser, Subcommand};
use cmd::{
    agent::AgentSubcommand, artifact::ArtifactSubcommand, comment::CommentSubcommand,
    config::ConfigSubcommand, feature::FeatureSubcommand, investigate::InvestigateSubcommand,
    milestone::MilestoneSubcommand, platform::PlatformSubcommand, ponder::PonderSubcommand,
    project::ProjectSubcommand, query::QuerySubcommand, score::ScoreSubcommand,
    task::TaskSubcommand, ui::UiSubcommand,
};
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "sdlc",
    about = "Deterministic SDLC state machine — manage features, artifacts, tasks, and milestones",
    version,
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

    /// Manage features
    Feature {
        #[command(subcommand)]
        subcommand: FeatureSubcommand,
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

    /// Refresh agent scaffolding and stamp the current binary version
    Update,

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

        #[command(subcommand)]
        subcommand: Option<UiSubcommand>,
    },
}

fn main() {
    let cli = Cli::parse();

    let default_level = match &cli.command {
        Commands::Ui { .. } | Commands::Mcp => tracing::Level::INFO,
        _ => tracing::Level::WARN,
    };

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env().add_directive(default_level.into()),
        )
        .with_target(false)
        .init();

    let root_path = cli.root.as_deref();
    let root = root::resolve_root(root_path);

    let result = match cli.command {
        Commands::Init { platform } => cmd::init::run(&root, platform.as_deref()),
        Commands::State => cmd::state::run(&root, cli.json),
        Commands::Next { feature } => cmd::next::run(&root, feature.as_deref(), cli.json),
        Commands::Focus => cmd::focus::run(&root, cli.json),
        Commands::Feature { subcommand } => cmd::feature::run(&root, subcommand, cli.json),
        Commands::Artifact { subcommand } => cmd::artifact::run(&root, subcommand, cli.json),
        Commands::Task { subcommand } => cmd::task::run(&root, subcommand, cli.json),
        Commands::Comment { subcommand } => cmd::comment::run(&root, subcommand, cli.json),
        Commands::Milestone { subcommand } => cmd::milestone::run(&root, subcommand, cli.json),
        Commands::Ponder { subcommand } => cmd::ponder::run(&root, subcommand, cli.json),
        Commands::Investigate { subcommand } => cmd::investigate::run(&root, subcommand, cli.json),
        Commands::Platform { subcommand } => cmd::platform::run(&root, subcommand, cli.json),
        Commands::Project { subcommand } => cmd::project::run(&root, subcommand, cli.json),
        Commands::Query { subcommand } => cmd::query::run(&root, subcommand, cli.json),
        Commands::Config { subcommand } => cmd::config::run(&root, subcommand, cli.json),
        Commands::Score { subcommand } => cmd::score::run(&root, subcommand, cli.json),
        Commands::Update => cmd::update::run(&root),
        Commands::Merge { slug } => cmd::merge::run(&root, &slug, cli.json),
        Commands::Archive { slug } => {
            cmd::feature::run(&root, FeatureSubcommand::Archive { slug }, cli.json)
        }
        Commands::Mcp => cmd::mcp::run(&root),
        Commands::Agent { subcommand } => cmd::agent::run(&root, subcommand, cli.json),
        Commands::Ui {
            port,
            no_open,
            subcommand,
        } => cmd::ui::run(&root, subcommand, port, no_open),
    };

    if let Err(e) = result {
        // Print the full error chain (anyhow's alternate Display)
        eprintln!("error: {e:#}");
        std::process::exit(1);
    }
}
