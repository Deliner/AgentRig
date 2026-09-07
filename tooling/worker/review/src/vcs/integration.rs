use super::{Backend, Kind, Repository, Settings, Source, external, mercurial, validate_revision};
use anyhow::{Context as _, Result, ensure};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::process::Command;

#[derive(Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct Merge {
    feature: String,
    base: String,
    candidate: String,
}

impl Merge {
    fn validate_external(&self, settings: &Settings) -> Result<()> {
        external::revision(self.base.clone())?;
        external::revision(self.candidate.clone())?;
        ensure!(
            self.feature.starts_with(&settings.prefix),
            "integration requires a {} branch",
            settings.prefix
        );
        Ok(())
    }
}

impl Source<'_> {
    pub fn integrate(
        &self,
        settings: &Settings,
        mut check: impl FnMut() -> Result<i32>,
        run: impl FnMut(&mut Command) -> Result<i32>,
    ) -> Result<(String, i32)> {
        match self.backend {
            Backend::Native(kind) => {
                Repository::new(self.root, *kind).integrate(settings, check, run)
            }
            Backend::External(adapter) => {
                let policy = json!({"base": settings.base, "prefix": settings.prefix});
                let merge: Merge =
                    adapter.call(self.root, "prepare-integration", policy.clone())?;
                merge.validate_external(settings)?;
                let code = check()?;
                let passed = code == 0;
                if passed {
                    adapter.call::<()>(
                        self.root,
                        "finish-integration",
                        json!({"policy": policy, "expected": merge}),
                    )?;
                    let current = self.observe()?;
                    ensure!(
                        current.branch == settings.base
                            && current.status.is_empty()
                            && !current.merge_in_progress
                            && !current.rebase_in_progress,
                        "external VCS did not finish clean integration on the base; inspect and preserve its state"
                    );
                }
                Ok((merge.feature, code))
            }
        }
    }
}

impl Repository<'_> {
    pub fn integrate(
        &self,
        settings: &Settings,
        mut check: impl FnMut() -> Result<i32>,
        run: impl FnMut(&mut Command) -> Result<i32>,
    ) -> Result<(String, i32)> {
        match self.kind {
            Kind::Git => self.integrate_git(settings, check, run),
            Kind::Mercurial => {
                let merge = self.prepare_mercurial_merge(settings)?;
                let code = check()?;
                let passed = code == 0;
                if passed {
                    self.commit_mercurial_merge(settings, &merge)?;
                }
                Ok((merge.feature, code))
            }
        }
    }

    fn prepare_mercurial_merge(&self, settings: &Settings) -> Result<Merge> {
        ensure!(
            self.kind == Kind::Mercurial,
            "native Mercurial integration required"
        );
        let observed = self.observe()?;
        ensure!(
            !observed.rebase_in_progress,
            "finish or abort the pending rebase before integration"
        );
        if observed.merge_in_progress {
            return self.mercurial_merge_context(settings);
        }
        ensure!(
            observed.branch.starts_with(&settings.prefix),
            "integration requires a {} branch",
            settings.prefix
        );
        ensure!(
            observed.status.is_empty(),
            "working tree must be clean; preserve or commit pending changes before integration"
        );
        let candidate = self.head()?.context("feature has no committed revision")?;
        ensure!(
            self.mercurial_revision_branch(&candidate)? == observed.branch,
            "feature has no committed changes; preserve its branch and commit before integration"
        );
        let base = self.mercurial_branch_head(&settings.base)?;
        mercurial::write(self.root, &["update", "--check", "--rev", &base])?;
        mercurial::write(
            self.root,
            &["merge", "--rev", &candidate, "--tool", "internal:merge"],
        )?;
        self.mercurial_merge_context(settings)
    }

    fn commit_mercurial_merge(&self, settings: &Settings, merge: &Merge) -> Result<()> {
        ensure!(
            self.mercurial_merge_context(settings)? == *merge,
            "merge parents changed during checks; inspect the native operation and retry"
        );
        let message = format!("Merge {}", merge.feature);
        mercurial::write(self.root, &["commit", "--message", &message])?;
        let head = self.head()?.context("merged revision required")?;
        ensure!(
            self.parents(&head)? == [merge.base.clone(), merge.candidate.clone()],
            "integration commit has unexpected parents; inspect the retained revisions"
        );
        ensure!(
            self.observe()?.status.is_empty(),
            "merge committed but working changes remain; preserve and inspect them"
        );
        Ok(())
    }

    fn mercurial_merge_context(&self, settings: &Settings) -> Result<Merge> {
        let parents = String::from_utf8(mercurial::run(
            self.root,
            &["parents", "--template", "{node}\n"],
        )?)?
        .lines()
        .map(|value| validate_revision(value.into()))
        .collect::<Result<Vec<_>>>()?;
        ensure!(
            parents.len() == 2,
            "integration requires a pending merge with two parents"
        );
        let observed = self.observe()?;
        ensure!(
            observed.branch == settings.base,
            "resume integration from base branch {}",
            settings.base
        );
        ensure!(
            parents[0] == self.mercurial_branch_head(&settings.base)?,
            "base advanced during integration; preserve the merge, abort it and retry from the feature"
        );
        let feature = self.mercurial_revision_branch(&parents[1])?;
        ensure!(
            feature.starts_with(&settings.prefix),
            "merge second parent must belong to a {} branch",
            settings.prefix
        );
        Ok(Merge {
            feature,
            base: parents[0].clone(),
            candidate: parents[1].clone(),
        })
    }

    fn mercurial_revision_branch(&self, revision: &str) -> Result<String> {
        Ok(String::from_utf8(mercurial::run(
            self.root,
            &["log", "--rev", revision, "--template", "{branch}"],
        )?)?)
    }

    fn mercurial_branch_head(&self, branch: &str) -> Result<String> {
        let escaped = branch.replace('\\', "\\\\").replace('\'', "\\'");
        let query = format!("heads(branch('{escaped}'))");
        self.resolve(&query).context(
            "base branch must have exactly one head; resolve multiple heads before integration",
        )
    }
}
