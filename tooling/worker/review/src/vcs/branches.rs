use super::{Kind, Repository, Settings, git, mercurial};
use anyhow::{Result, ensure};

impl Settings {
    pub fn validate(&self) -> Result<()> {
        ensure!(
            self.backend.branch_name(&self.base),
            "vcs.base must name a valid {} branch without normalization",
            self.backend.executable()
        );
        ensure!(
            !self.prefix.is_empty()
                && self.backend.branch_name(&format!("{}example", self.prefix))
                && !self.base.starts_with(&self.prefix),
            "vcs.prefix must form valid {} branches distinct from vcs.base",
            self.backend.executable()
        );
        Ok(())
    }
}

impl Kind {
    fn branch_name(self, value: &str) -> bool {
        match self {
            Self::Git => git::branch_name(value),
            Self::Mercurial => mercurial::branch_name(value),
        }
    }
}

impl Repository<'_> {
    pub fn start_feature(&self, settings: &Settings, name: &str) -> Result<String> {
        ensure!(
            !name.is_empty() && !name.starts_with('-'),
            "feature name required"
        );
        let branch = format!("{}{name}", settings.prefix);
        let observed = self.observe()?;
        ensure!(
            observed.branch == settings.base,
            "start from {}",
            settings.base
        );
        ensure!(
            observed.status.is_empty(),
            "working tree must be clean; preserve or commit pending changes before switching branches"
        );
        ensure!(
            !observed.merge_in_progress && !observed.rebase_in_progress,
            "finish or abort the pending VCS operation before starting a feature"
        );
        match self.kind {
            Kind::Git => {
                git::run(self.root, &["check-ref-format", "--branch", &branch])?;
                git::run(self.root, &["switch", "-c", &branch])?;
            }
            Kind::Mercurial => mercurial::create_branch(self.root, &branch)?,
        }
        Ok(branch)
    }
}
