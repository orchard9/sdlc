use anyhow::Context;
use sdlc_core::{
    classifier::{Classification, Classifier, EvalContext},
    config::{AgentBackend, Config},
    feature::Feature,
    gate::GateKind,
    paths,
    rules::default_rules,
    state::State,
    types::ActionType,
};
use std::path::Path;

use crate::cmd::gate::run_gates;

// ---------------------------------------------------------------------------
// RunExit — typed non-zero exit codes (no std::process::exit in library code)
// ---------------------------------------------------------------------------

#[derive(Debug)]
pub enum RunExit {
    AgentFailed(i32),
    GateFailed { gate_name: String, attempts: u32 },
    HumanGateRequired { gate_name: String },
}

impl RunExit {
    pub fn exit_code(&self) -> i32 {
        match self {
            RunExit::AgentFailed(c) => *c,
            RunExit::GateFailed { .. } => 2,
            RunExit::HumanGateRequired { .. } => 3,
        }
    }
}

impl std::fmt::Display for RunExit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RunExit::AgentFailed(code) => {
                write!(f, "agent subprocess exited with code {code}")
            }
            RunExit::GateFailed {
                gate_name,
                attempts,
            } => {
                write!(f, "gate '{gate_name}' failed after {attempts} attempt(s)")
            }
            RunExit::HumanGateRequired { gate_name } => {
                write!(f, "human gate '{gate_name}' requires approval")
            }
        }
    }
}

impl std::error::Error for RunExit {}

// ---------------------------------------------------------------------------
// run
// ---------------------------------------------------------------------------

pub fn run(root: &Path, slug: &str, dry_run: bool) -> anyhow::Result<()> {
    let config = Config::load(root).context("failed to load config")?;
    let state = State::load(root).context("failed to load state")?;
    let feature =
        Feature::load(root, slug).with_context(|| format!("feature '{slug}' not found"))?;

    let ctx = EvalContext {
        feature: &feature,
        state: &state,
        config: &config,
        root,
    };

    let classifier = Classifier::new(default_rules());
    let classification = classifier.classify(&ctx);

    // Done — no action needed
    if classification.action == ActionType::Done {
        println!("Feature '{}' is complete — no pending actions.", slug);
        return Ok(());
    }

    let backend = config.agents.backend_for(classification.action);

    // Human gate or wait actions — exit without subprocess
    if matches!(backend, AgentBackend::Human)
        || matches!(
            classification.action,
            ActionType::WaitForApproval | ActionType::ApproveMerge
        )
    {
        println!(
            "Human action required for '{}'.\n{}",
            slug, classification.message
        );
        if !classification.next_command.is_empty() {
            println!("Next command:  {}", classification.next_command);
        }
        return Ok(());
    }

    let context_str = build_context_str(&classification, root);
    let argv = build_argv(backend, &context_str);

    let gates = config.gates_for(classification.action.as_str());

    if dry_run {
        println!("{}", argv.join(" "));
        if !gates.is_empty() {
            println!("\nGates after {}:", classification.action);
            for gate in gates {
                match &gate.gate_type {
                    GateKind::Shell { command } => {
                        println!(
                            "  shell: {} (retries: {}, timeout: {}s)",
                            gate.name, gate.max_retries, gate.timeout_seconds
                        );
                        println!("    command: {command}");
                    }
                    GateKind::Human { prompt } => {
                        println!("  human: {} — {}", gate.name, prompt);
                    }
                    GateKind::StepBack { questions } => {
                        println!("  step_back: {} ({} questions)", gate.name, questions.len());
                        for q in questions {
                            println!("    - {q}");
                        }
                    }
                }
            }
        }
        return Ok(());
    }

    let status = std::process::Command::new(&argv[0])
        .args(&argv[1..])
        .status()
        .with_context(|| format!("failed to execute '{}'", argv[0]))?;

    if !status.success() {
        return Err(RunExit::AgentFailed(status.code().unwrap_or(1)).into());
    }

    // Run verification gates if configured for this action
    if !gates.is_empty() {
        eprintln!(
            "\nRunning verification gates for '{}'...",
            classification.action
        );
        let results = run_gates(root, classification.action.as_str(), gates);

        for result in &results {
            if result.passed {
                eprintln!(
                    "  \u{2713} {} (attempt {}, {}ms)",
                    result.gate_name, result.attempt, result.duration_ms
                );
            } else {
                eprintln!(
                    "  \u{2717} {} (attempt {}, {}ms)",
                    result.gate_name, result.attempt, result.duration_ms
                );
                if !result.output.is_empty() {
                    for line in result.output.lines() {
                        eprintln!("    {line}");
                    }
                }
            }
        }

        if !results.iter().all(|r| r.passed) {
            // The last result is always the failing one (run_gates stops on failure)
            if let Some(failing) = results.last().filter(|r| !r.passed) {
                if let Some(gate_def) = gates.iter().find(|g| g.name == failing.gate_name) {
                    match &gate_def.gate_type {
                        GateKind::Human { .. } | GateKind::StepBack { .. } => {
                            eprintln!(
                                "\nHuman gate '{}' requires approval.",
                                failing.gate_name
                            );
                            return Err(RunExit::HumanGateRequired {
                                gate_name: failing.gate_name.clone(),
                            }
                            .into());
                        }
                        GateKind::Shell { .. } => {
                            eprintln!(
                                "\nGate '{}' failed after {} attempt(s).",
                                failing.gate_name, failing.attempt
                            );
                            return Err(RunExit::GateFailed {
                                gate_name: failing.gate_name.clone(),
                                attempts: failing.attempt,
                            }
                            .into());
                        }
                    }
                }
            }
            return Err(RunExit::GateFailed {
                gate_name: "unknown".to_string(),
                attempts: 0,
            }
            .into());
        }
        eprintln!("All gates passed.");
    }

    Ok(())
}

fn build_context_str(c: &Classification, root: &Path) -> String {
    let mut parts = vec![
        format!("Feature: {}", c.feature),
        format!("Title: {}", c.title),
    ];
    if let Some(ref desc) = c.description {
        parts.push(format!("Description: {desc}"));
    }
    parts.push(format!("Phase: {}", c.current_phase));
    parts.push(format!("Action: {}", c.action));
    parts.push(c.message.clone());
    if !c.next_command.is_empty() {
        parts.push(format!("Next command: {}", c.next_command));
    }
    if let Some(ref path) = c.output_path {
        parts.push(format!("Output path: {path}"));
    }

    // Append VISION.md content if it exists
    let vision_path = paths::vision_md_path(root);
    if let Ok(vision) = std::fs::read_to_string(&vision_path) {
        if !vision.trim().is_empty() {
            parts.push(String::new());
            parts.push("--- VISION.md ---".to_string());
            parts.push(vision);
        }
    }

    parts.join("\n")
}

fn build_argv(backend: &AgentBackend, context: &str) -> Vec<String> {
    match backend {
        AgentBackend::Xadk { agent_id, .. } => vec![
            "python".to_string(),
            "-m".to_string(),
            "xadk".to_string(),
            agent_id.clone(),
            "--prompt".to_string(),
            context.to_string(),
        ],
        AgentBackend::ClaudeAgentSdk {
            model,
            allowed_tools,
            permission_mode,
            timeout_minutes,
        } => {
            let mut argv = vec![
                "claude".to_string(),
                "-p".to_string(),
                context.to_string(),
                "--model".to_string(),
                model.clone(),
            ];
            if !allowed_tools.is_empty() {
                argv.push("--allowedTools".to_string());
                argv.push(allowed_tools.join(","));
            }
            if let Some(mode) = permission_mode {
                argv.push("--permission-mode".to_string());
                argv.push(mode.clone());
            }
            if let Some(t) = timeout_minutes {
                argv.push("--timeout".to_string());
                argv.push(format!("{}", t * 60));
            }
            argv
        }
        // Human is handled before build_argv is called
        AgentBackend::Human => {
            unreachable!("Human backend is handled before build_argv is called")
        }
    }
}
