use crate::output::print_json;
use anyhow::Context;
use sdlc_core::{
    config::Config,
    feature::Feature,
    state::State,
    types::{ActionType, Phase},
};
use std::path::Path;

pub fn run(root: &Path, slug: &str, json: bool) -> anyhow::Result<()> {
    let config = Config::load(root).context("failed to load config")?;
    let mut feature =
        Feature::load(root, slug).with_context(|| format!("feature '{slug}' not found"))?;

    if feature.phase != Phase::Merge {
        anyhow::bail!(
            "cannot finalize merge for '{slug}' from phase '{}'; move it to 'merge' first",
            feature.phase
        );
    }

    feature
        .transition(Phase::Released, &config)
        .with_context(|| format!("cannot transition '{slug}' to released"))?;
    feature.save(root).context("failed to save feature")?;

    let mut state = State::load(root).context("failed to load state")?;
    state.record_action(slug, ActionType::Merge, Phase::Released, "merged");
    state.complete_directive(slug);
    state.save(root).context("failed to save state")?;

    if json {
        print_json(&serde_json::json!({
            "slug": slug,
            "phase": "released",
            "merged": true,
        }))?;
    } else {
        println!("Merged '{slug}' and marked as released");
    }

    Ok(())
}
