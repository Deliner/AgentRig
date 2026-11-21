use super::pid_reuse_guard::Identity;
use super::record::{Grouping, Record};
use super::{storage, streams};
use crate::project_config::{Access, Context, Lifetime};
use crate::shared::fs_with_context::{self as fs, OpenOptions};
use anyhow::Result;
use std::{
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

pub struct Job {
    pub(super) directory: PathBuf,
    pub(super) record: Record,
    pub(super) started: std::time::Instant,
}
impl Job {
    pub fn create(context: &Context, command: &str, cwd: &Path) -> Result<Self> {
        let project = &context.root;
        let directory = storage::create(&context.runtime)?;
        let run_id = directory
            .file_name()
            .ok_or_else(|| anyhow::anyhow!("job directory has no name"))?
            .to_string_lossy()
            .into_owned();
        let owner = owner().unwrap_or_else(|| run_id.clone());
        let branch = crate::shared::vcs::git_context(project, &["branch", "--show-current"])
            .ok()
            .filter(|value| !value.is_empty());
        let record = Record {
            run_id,
            owner,
            parent_run: std::env::var("WORKER_PARENT_RUN").ok(),
            project: project.into(),
            cwd: cwd.into(),
            branch,
            command: command.into(),
            argv: Vec::new(),
            started: now()?,
            finished: None,
            exit_code: None,
            error: None,
            supervisor: Identity::read(std::process::id())?,
            child: None,
            grouping: Grouping::Inherited,
            duration_seconds: None,
            group: None,
            access: Access::Unconfined,
            lifetime: Lifetime::Task,
            background: false,
        };
        Self::registered(directory, record)
    }
    fn registered(directory: PathBuf, record: Record) -> Result<Self> {
        let job = Self {
            directory,
            record,
            started: std::time::Instant::now(),
        };
        job.save()?;
        Ok(job)
    }
    pub fn prepare(&mut self, argv: &[String], access: Access, lifetime: Lifetime) -> Result<()> {
        self.record.argv = argv.into();
        self.record.access = access;
        self.record.lifetime = lifetime;
        self.save()
    }
    pub fn directory(&self) -> &Path {
        &self.directory
    }
    pub fn attach(&mut self, pid: u32, grouping: Grouping) -> Result<()> {
        self.record.child = Identity::read(pid).ok();
        self.record.grouping = grouping;
        self.save()
    }
    pub fn finish(&mut self, code: i32, error: Option<String>) -> Result<()> {
        self.record.duration_seconds = Some(self.started.elapsed().as_secs_f64());
        self.record.finished = Some(now()?);
        self.record.exit_code = Some(code);
        self.record.error = error;
        self.save()
    }
    pub(super) fn save(&self) -> Result<()> {
        storage::save(&self.directory, &self.record)
    }
    pub(super) fn open_logs(&self) -> Result<streams::Logs> {
        let open = |name: &str| {
            fs::open_with(
                OpenOptions::new().create_new(true).write(true),
                self.directory.join(name),
            )
        };
        Ok(streams::Logs {
            stdout: open("stdout.log")?,
            stderr: open("stderr.log")?,
        })
    }
}
pub fn owner() -> Option<String> {
    ["WORKER_OWNER", "CODEX_THREAD_ID", "CODEX_SESSION_ID"]
        .iter()
        .find_map(|key| {
            std::env::var(key)
                .ok()
                .filter(|value| !value.trim().is_empty())
        })
}
pub(super) fn now() -> Result<u64> {
    Ok(SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs())
}
