use crate::output::print_json;
use anyhow::Context;
use sdlc_core::focus::focus;
use std::path::Path;

pub fn run(root: &Path, json: bool) -> anyhow::Result<()> {
    let result = focus(root).context("failed to determine focus")?;

    match result {
        None => {
            if json {
                print_json(&serde_json::Value::Null)?;
            } else {
                println!("Nothing to work on. All features are done, blocked, or waiting.");
            }
        }
        Some(r) => {
            if json {
                print_json(&r)?;
            } else {
                print!(
                    "Next: {} [{}] → {}",
                    r.classification.feature,
                    r.classification.current_phase,
                    r.classification.action
                );
                if let Some(ref ms) = r.milestone {
                    println!("  ({}  {}/{})", ms.title, ms.position, ms.total);
                } else {
                    println!();
                }
                println!("  {}", r.classification.message);
                if !r.classification.next_command.is_empty() {
                    println!("  Run: {}", r.classification.next_command);
                }
            }
        }
    }

    Ok(())
}
