/// A slash command definition across all target platforms.
pub struct CommandDef {
    pub slug: &'static str,
    pub claude_content: &'static str,
    pub gemini_description: &'static str,
    pub playbook: &'static str, // shared Gemini + OpenCode
    pub opencode_description: &'static str,
    pub opencode_hint: &'static str,
    pub skill: &'static str,
}

impl CommandDef {
    pub fn claude_filename(&self) -> String {
        format!("{}.md", self.slug)
    }
    pub fn gemini_filename(&self) -> String {
        format!("{}.toml", self.slug)
    }
    pub fn opencode_filename(&self) -> String {
        format!("{}.md", self.slug)
    }
    pub fn skill_dirname(&self) -> String {
        self.slug.to_string()
    }
}
