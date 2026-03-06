mod sdlc_approve;
mod sdlc_architecture_adjustment;
mod sdlc_beat;
mod sdlc_convo_mine;
mod sdlc_cookbook;
mod sdlc_cookbook_run;
mod sdlc_empathy;
mod sdlc_enterprise_readiness;
mod sdlc_fit_impact;
mod sdlc_guideline;
mod sdlc_hypothetical_do;
mod sdlc_hypothetical_planning;
mod sdlc_init;
mod sdlc_knowledge;
mod sdlc_milestone_uat;
mod sdlc_next;
mod sdlc_organize_parallel;
mod sdlc_plan;
mod sdlc_ponder;
mod sdlc_ponder_commit;
mod sdlc_prepare;
mod sdlc_pressure_test;
mod sdlc_quality_fix;
mod sdlc_recap;
mod sdlc_recruit;
mod sdlc_run;
mod sdlc_run_wave;
mod sdlc_setup_quality_gates;
mod sdlc_skill_build;
mod sdlc_specialize;
mod sdlc_spike;
mod sdlc_status;
mod sdlc_suggest;
mod sdlc_tool_audit;
mod sdlc_tool_build;
mod sdlc_tool_run;
mod sdlc_tool_uat;
mod sdlc_vision_adjustment;

use crate::cmd::init::registry::CommandDef;

/// All commands in canonical order (matches write_user_claude_commands ordering).
pub static ALL_COMMANDS: &[&CommandDef] = &[
    &sdlc_next::SDLC_NEXT,
    &sdlc_status::SDLC_STATUS,
    &sdlc_approve::SDLC_APPROVE,
    &sdlc_specialize::SDLC_SPECIALIZE,
    &sdlc_run::SDLC_RUN,
    &sdlc_plan::SDLC_PLAN,
    &sdlc_milestone_uat::SDLC_MILESTONE_UAT,
    &sdlc_pressure_test::SDLC_PRESSURE_TEST,
    &sdlc_enterprise_readiness::SDLC_ENTERPRISE_READINESS,
    &sdlc_setup_quality_gates::SDLC_SETUP_QUALITY_GATES,
    &sdlc_cookbook::SDLC_COOKBOOK,
    &sdlc_cookbook_run::SDLC_COOKBOOK_RUN,
    &sdlc_ponder::SDLC_PONDER,
    &sdlc_ponder_commit::SDLC_PONDER_COMMIT,
    &sdlc_suggest::SDLC_SUGGEST,
    &sdlc_beat::SDLC_BEAT,
    &sdlc_fit_impact::SDLC_FIT_IMPACT,
    &sdlc_recruit::SDLC_RECRUIT,
    &sdlc_empathy::SDLC_EMPATHY,
    &sdlc_prepare::SDLC_PREPARE,
    &sdlc_run_wave::SDLC_RUN_WAVE,
    &sdlc_organize_parallel::SDLC_ORGANIZE_PARALLEL,
    &sdlc_tool_run::SDLC_TOOL_RUN,
    &sdlc_tool_build::SDLC_TOOL_BUILD,
    &sdlc_skill_build::SDLC_SKILL_BUILD,
    &sdlc_tool_audit::SDLC_TOOL_AUDIT,
    &sdlc_tool_uat::SDLC_TOOL_UAT,
    &sdlc_quality_fix::SDLC_QUALITY_FIX,
    &sdlc_vision_adjustment::SDLC_VISION_ADJUSTMENT,
    &sdlc_architecture_adjustment::SDLC_ARCHITECTURE_ADJUSTMENT,
    &sdlc_guideline::SDLC_GUIDELINE,
    &sdlc_knowledge::SDLC_KNOWLEDGE,
    &sdlc_hypothetical_planning::SDLC_HYPOTHETICAL_PLANNING,
    &sdlc_hypothetical_do::SDLC_HYPOTHETICAL_DO,
    &sdlc_spike::SDLC_SPIKE,
    &sdlc_init::SDLC_INIT,
    &sdlc_convo_mine::SDLC_CONVO_MINE,
    &sdlc_recap::SDLC_RECAP,
];
