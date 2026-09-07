// DECISION: D010
use super::{Repository, Settings, git};
use anyhow::{Result, ensure};
use std::process::Command;

impl Repository<'_> {
    pub(super) fn integrate_git(
        &self,
        settings: &Settings,
        mut check: impl FnMut() -> Result<i32>,
        mut run: impl FnMut(&mut Command) -> Result<i32>,
    ) -> Result<(String, i32)> {
        let feature = self.observe()?.branch;
        ensure!(
            feature.starts_with(&settings.prefix),
            "integration requires a {} branch",
            settings.prefix
        );
        self.git_clean()?;
        let code = self.prepare_git(settings, &feature, &mut run)?;
        let failed = code != 0;
        if failed {
            return Ok((feature, code));
        }
        let code = check()?;
        let failed = code != 0;
        if failed {
            return Ok((feature, code));
        }
        self.git_clean()?;
        self.select_git_base(settings, &feature)?;
        let code = run(&mut git::command(
            self.root,
            &["merge", "--no-ff", "--no-edit", &feature],
        ))?;
        Ok((feature, code))
    }

    fn prepare_git(
        &self,
        settings: &Settings,
        feature: &str,
        run: &mut impl FnMut(&mut Command) -> Result<i32>,
    ) -> Result<i32> {
        let divergent = !git::ancestor(self.root, &settings.base, feature)?;
        if divergent {
            return run(&mut git::command(
                self.root,
                &["rebase", "--rebase-merges", &settings.base],
            ));
        }
        Ok(0)
    }

    fn select_git_base(&self, settings: &Settings, feature: &str) -> Result<()> {
        let base = self.resolve(&settings.base)?;
        git::run(self.root, &["switch", &settings.base])?;
        let unchanged = self.resolve(&settings.base)? == base
            && git::ancestor(self.root, &settings.base, feature)?;
        let base_advanced = !unchanged;
        if base_advanced {
            git::run(self.root, &["switch", feature])?;
            anyhow::bail!("base advanced during integration; retry from the feature branch");
        }
        Ok(())
    }

    fn git_clean(&self) -> Result<()> {
        ensure!(
            git::run(self.root, &["status", "--porcelain"])?.is_empty(),
            "working tree must be clean; preserve or commit pending changes before switching branches"
        );
        Ok(())
    }
}
