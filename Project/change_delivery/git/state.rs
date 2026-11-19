// DECISION: D191
use crate::project_config::Context;
use crate::project_memory::format::{reset_state, state_holds_notes, state_template};
use crate::shared::console;
use crate::shared::vcs::Repository;
use anyhow::{Result, ensure};
use std::path::PathBuf;

pub(super) struct State {
    path: String,
    file: PathBuf,
    template: String,
    skill: PathBuf,
}

impl State {
    pub(super) fn of(context: &Context) -> Result<Self> {
        let memory = &context.config.paths.memory;
        Ok(Self {
            path: format!("{memory}/State.md"),
            file: context.path(memory)?.join("State.md"),
            template: state_template(),
            skill: context
                .path(&context.config.paths.skills)?
                .join("edit-state/SKILL.md"),
        })
    }

    pub(super) fn guard(&self, repository: &Repository<'_>) -> Result<i32> {
        let staged = repository.run(&["diff", "--cached", "--name-only", "-z"])?;
        let state = self.path.as_bytes();
        let carries_work = staged
            .split(|byte| *byte == 0)
            .any(|path| !path.is_empty() && path != state);
        let recorded = repository.run(&["show", &format!(":{}", self.path)])?;
        let described = recorded != self.template.as_bytes();
        let accepted = described || !carries_work;
        if accepted {
            return Ok(0);
        }
        console::stderr(format!(
            "{path} is the empty State template; describe the VAC of this commit in it as {skill} directs",
            path = self.file.display(),
            skill = self.skill.display()
        ));
        Ok(1)
    }

    pub(super) fn reset(&self, repository: &Repository<'_>) -> Result<()> {
        reset_state(&self.file)?;
        repository.run(&["add", "--", &self.path])?;
        Ok(())
    }

    pub(super) fn excluded(&self) -> String {
        format!(":(exclude){}", self.path)
    }

    pub(super) fn commit_reset(
        &self,
        repository: &Repository<'_>,
        mut run: impl FnMut(&[&str]) -> Result<i32>,
    ) -> Result<i32> {
        let committed = repository.run(&["show", &format!("HEAD:{}", self.path)])?;
        let notes = state_holds_notes(&self.file, &committed)?;
        ensure!(
            !notes,
            "{} holds uncommitted notes; commit them with their VAC or restore State before integration",
            self.file.display()
        );
        self.reset(repository)?;
        let already_reset = committed == self.template.as_bytes();
        if already_reset {
            return Ok(0);
        }
        run(&["commit", "-qm", "Reset State before integration"])
    }
}
