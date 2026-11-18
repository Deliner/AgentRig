// DECISION: D188
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Agent {
    Codex,
    ClaudeCode,
}
impl Agent {
    pub const ALL: [Self; 2] = [Self::Codex, Self::ClaudeCode];
    pub fn skills(self) -> &'static str {
        match self {
            Self::Codex => ".agents/skills",
            Self::ClaudeCode => ".claude/skills",
        }
    }
}
