// DECISION: D188
use anyhow::Result;
use std::path::{Path, PathBuf};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Agent {
    Codex,
    ClaudeCode,
}
impl Agent {
    pub const ALL: [Self; 2] = [Self::Codex, Self::ClaudeCode];
    pub fn name(self) -> &'static str {
        match self {
            Self::Codex => "codex",
            Self::ClaudeCode => "claude-code",
        }
    }
    pub fn skills(self) -> &'static str {
        match self {
            Self::Codex => ".agents/skills",
            Self::ClaudeCode => ".claude/skills",
        }
    }
    pub fn parse(name: &str) -> Result<Self> {
        Self::ALL
            .into_iter()
            .find(|agent| agent.name() == name)
            .ok_or_else(|| {
                let names = Self::ALL.map(Self::name);
                anyhow::anyhow!("--agent expects {}, not {name:?}", names.join(" or "))
            })
    }
}

#[derive(Clone, Debug)]
pub struct Caller {
    pub root: PathBuf,
    pub skills: PathBuf,
    pub agent: Option<Agent>,
}
impl Caller {
    pub fn skill(&self, configured: &str) -> PathBuf {
        let configured = Path::new(configured);
        let through = self.agent.and_then(|agent| {
            let inner = configured.strip_prefix(&self.skills).ok()?;
            Some(Path::new(agent.skills()).join(inner))
        });
        self.root.join(through.as_deref().unwrap_or(configured))
    }
}
