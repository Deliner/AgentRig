use super::{Repository, Settings, git};
use anyhow::{Result, ensure};

impl Settings {
    pub fn validate(&self) -> Result<()> {
        ensure!(
            git::branch_name(&self.base),
            "vcs.base must name a valid git branch without normalization"
        );
        ensure!(
            !self.prefix.is_empty()
                && git::branch_name(&format!("{}example", self.prefix))
                && !self.base.starts_with(&self.prefix),
            "vcs.prefix must form valid git branches distinct from vcs.base"
        );
        Ok(())
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
        git::run(self.root, &["check-ref-format", "--branch", &branch])?;
        git::run(self.root, &["switch", "-c", &branch])?;
        Ok(branch)
    }
}
