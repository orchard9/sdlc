use crate::config::Config;
use crate::feature::Feature;
use crate::state::State;
use crate::types::{ActionType, Phase};
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// EvalContext
// ---------------------------------------------------------------------------

pub struct EvalContext<'a> {
    pub feature: &'a Feature,
    pub state: &'a State,
    pub config: &'a Config,
    pub root: &'a std::path::Path,
}

// ---------------------------------------------------------------------------
// Classification (output)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Classification {
    pub feature: String,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub current_phase: Phase,
    pub action: ActionType,
    pub message: String,
    pub next_command: String,
    pub output_path: Option<String>,
    pub transition_to: Option<Phase>,
    pub task_id: Option<String>,
    /// Advisory hint for directive consumers: true if this action is
    /// resource-intensive. Included in directive output as consumer metadata.
    pub is_heavy: bool,
    /// Advisory hint for directive consumers: suggested timeout budget in
    /// minutes. Included in directive output as consumer metadata.
    pub timeout_minutes: u32,
}

// ---------------------------------------------------------------------------
// Rule
// ---------------------------------------------------------------------------

/// A fn-pointer rule — zero-cost, no heap allocation.
pub struct Rule {
    pub id: &'static str,
    pub condition: fn(&EvalContext) -> bool,
    pub action: ActionType,
    pub message: fn(&EvalContext) -> String,
    pub next_command: fn(&EvalContext) -> String,
    pub output_path: Option<fn(&EvalContext) -> String>,
    pub transition_to: Option<Phase>,
    pub task_id: Option<fn(&EvalContext) -> String>,
}

// ---------------------------------------------------------------------------
// Classifier
// ---------------------------------------------------------------------------

pub struct Classifier {
    rules: Vec<Rule>,
}

impl Classifier {
    pub fn new(rules: Vec<Rule>) -> Self {
        Self { rules }
    }

    pub fn classify(&self, ctx: &EvalContext) -> Classification {
        for rule in &self.rules {
            if (rule.condition)(ctx) {
                return Classification {
                    feature: ctx.feature.slug.clone(),
                    title: ctx.feature.title.clone(),
                    description: ctx.feature.description.clone(),
                    current_phase: ctx.feature.phase,
                    action: rule.action,
                    message: (rule.message)(ctx),
                    next_command: (rule.next_command)(ctx),
                    output_path: rule.output_path.map(|f| f(ctx)),
                    transition_to: rule.transition_to,
                    task_id: rule.task_id.map(|f| f(ctx)),
                    is_heavy: rule.action.is_heavy(),
                    timeout_minutes: rule.action.timeout_minutes(),
                };
            }
        }

        // Fallback: done
        Classification {
            feature: ctx.feature.slug.clone(),
            title: ctx.feature.title.clone(),
            description: ctx.feature.description.clone(),
            current_phase: ctx.feature.phase,
            action: ActionType::Done,
            message: format!("Feature '{}' has no pending actions", ctx.feature.slug),
            next_command: String::new(),
            output_path: None,
            transition_to: None,
            task_id: None,
            is_heavy: false,
            timeout_minutes: 0,
        }
    }
}
