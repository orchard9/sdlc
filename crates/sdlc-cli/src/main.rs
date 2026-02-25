mod cmd;
mod output;
mod root;

use clap::{Parser, Subcommand};
use cmd::{
    artifact::ArtifactSubcommand, comment::CommentSubcommand, config::ConfigSubcommand,
    feature::FeatureSubcommand, milestone::MilestoneSubcommand, platform::PlatformSubcommand,
    project::ProjectSubcommand, query::QuerySubcommand, score::ScoreSubcommand,
    task::TaskSubcommand,
};
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "sdlc",
    about = "Deterministic SDLC orchestration",
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

    /// Inspect and modify agent backend routing
    Config {
        #[command(subcommand)]
        subcommand: ConfigSubcommand,
    },

    /// Manage quality scores on features
    Score {
        #[command(subcommand)]
        subcommand: ScoreSubcommand,
    },

    /// Run the next SDLC step for a feature using the configured backend
    Run {
        slug: String,
        /// Print the command that would be run without executing it
        #[arg(long)]
        dry_run: bool,
    },

    /// Merge a feature (stub)
    Merge { slug: String },

    /// Archive a feature
    Archive { slug: String },

    /// Launch the web UI
    Ui {
        /// Port to listen on
        #[arg(long, default_value = "3141")]
        port: u16,

        /// Don't open browser automatically
        #[arg(long)]
        no_open: bool,
    },
}

fn main() {
    let cli = Cli::parse();

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::WARN.into()),
        )
        .with_target(false)
        .init();

    let root_path = cli.root.as_deref();
    let root = root::resolve_root(root_path);

    let result = match cli.command {
        Commands::Init { platform } => cmd::init::run(&root, platform.as_deref()),
        Commands::State => cmd::state::run(&root, cli.json),
        Commands::Next { feature } => cmd::next::run(&root, feature.as_deref(), cli.json),
        Commands::Feature { subcommand } => cmd::feature::run(&root, subcommand, cli.json),
        Commands::Artifact { subcommand } => cmd::artifact::run(&root, subcommand, cli.json),
        Commands::Task { subcommand } => cmd::task::run(&root, subcommand, cli.json),
        Commands::Comment { subcommand } => cmd::comment::run(&root, subcommand, cli.json),
        Commands::Milestone { subcommand } => cmd::milestone::run(&root, subcommand, cli.json),
        Commands::Platform { subcommand } => cmd::platform::run(&root, subcommand, cli.json),
        Commands::Project { subcommand } => cmd::project::run(&root, subcommand, cli.json),
        Commands::Query { subcommand } => cmd::query::run(&root, subcommand, cli.json),
        Commands::Config { subcommand } => cmd::config::run(&root, subcommand, cli.json),
        Commands::Score { subcommand } => cmd::score::run(&root, subcommand, cli.json),
        Commands::Run { slug, dry_run } => cmd::run::run(&root, &slug, dry_run),
        Commands::Merge { slug } => {
            eprintln!("merge not yet implemented (slug: {slug})");
            Ok(())
        }
        Commands::Archive { slug } => {
            cmd::feature::run(&root, FeatureSubcommand::Archive { slug }, cli.json)
        }
        Commands::Ui { port, no_open } => {
            let rt = tokio::runtime::Runtime::new().expect("failed to create tokio runtime");
            rt.block_on(sdlc_server::serve(root.clone(), port, !no_open))
                .map_err(|e| anyhow::anyhow!("{e}"))
        }
    };

    if let Err(e) = result {
        // RunExit carries specific exit codes (1=agent, 2=gate, 3=human gate)
        if let Some(run_exit) = e.downcast_ref::<cmd::run::RunExit>() {
            eprintln!("error: {run_exit}");
            std::process::exit(run_exit.exit_code());
        }
        eprintln!("error: {e}");
        std::process::exit(1);
    }
}
