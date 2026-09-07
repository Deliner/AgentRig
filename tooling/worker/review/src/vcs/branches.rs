use super::{Kind, Repository, Settings, git, mercurial};
use anyhow::{Result, ensure};

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
