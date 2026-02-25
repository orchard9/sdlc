use crate::output::print_json;
use anyhow::Context;
use sdlc_core::{
    classifier::{Classifier, EvalContext},
    config::Config,
    feature::Feature,
    rules::default_rules,
    state::State,
};
use std::path::Path;

pub fn run(root: &Path, feature_slug: Option<&str>, json: bool) -> anyhow::Result<()> {
    let config = Config::load(root).context("failed to load config")?;
    let state = State::load(root).context("failed to load state")?;
    let classifier = Classifier::new(default_rules());

    match feature_slug {
        Some(slug) => {
            let feature =
                Feature::load(root, slug).with_context(|| format!("feature '{slug}' not found"))?;
            let ctx = EvalContext {
                feature: &feature,
                state: &state,
                config: &config,
                root,
            };
            let classification = classifier.classify(&ctx);

            if json {
                print_json(&classification)?;
            } else {
                println!("Feature:  {}", classification.feature);
                println!("Phase:    {}", classification.current_phase);
                println!("Action:   {}", classification.action);
                println!("Message:  {}", classification.message);
                if !classification.next_command.is_empty() {
                    println!("Command:  {}", classification.next_command);
                }
                if let Some(ref path) = classification.output_path {
                    println!("Output:   {path}");
                }
            }
        }
        None => {
            // Classify all active features
            let features = Feature::list(root).context("failed to list features")?;
            let active: Vec<&Feature> = features.iter().filter(|f| !f.archived).collect();

            if active.is_empty() {
                println!("No active features. Run: sdlc feature create <slug>");
                return Ok(());
            }

            if json {
                let classifications: Vec<_> = active
                    .iter()
                    .map(|f| {
                        let ctx = EvalContext {
                            feature: f,
                            state: &state,
                            config: &config,
                            root,
                        };
                        classifier.classify(&ctx)
                    })
                    .collect();
                print_json(&classifications)?;
            } else {
                for feature in active {
                    let ctx = EvalContext {
                        feature,
                        state: &state,
                        config: &config,
                        root,
                    };
                    let c = classifier.classify(&ctx);
                    println!(
                        "{:<20} [{:<15}] {} — {}",
                        c.feature,
                        c.current_phase.to_string(),
                        c.action,
                        c.message
                    );
                }
            }
        }
    }

    Ok(())
}
