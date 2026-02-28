use std::path::Path;

pub mod add_comment;
pub mod add_task;
pub mod approve_artifact;
pub mod complete_task;
pub mod get_directive;
pub mod merge;
pub mod ponder_chat;
pub mod prepare;
pub mod project_phase;
pub mod reject_artifact;
pub mod run_wave;
pub mod write_artifact;

pub trait SdlcTool: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn schema(&self) -> serde_json::Value;
    fn call(&self, args: serde_json::Value, root: &Path) -> Result<serde_json::Value, String>;
}

pub fn all_tools() -> Vec<Box<dyn SdlcTool>> {
    vec![
        Box::new(get_directive::GetDirectiveTool),
        Box::new(write_artifact::WriteArtifactTool),
        Box::new(approve_artifact::ApproveArtifactTool),
        Box::new(reject_artifact::RejectArtifactTool),
        Box::new(add_task::AddTaskTool),
        Box::new(complete_task::CompleteTaskTool),
        Box::new(add_comment::AddCommentTool),
        Box::new(merge::MergeTool),
        Box::new(project_phase::ProjectPhaseTool),
        Box::new(prepare::PrepareTool),
        Box::new(run_wave::RunWaveTool),
        Box::new(ponder_chat::PonderChatTool),
    ]
}
