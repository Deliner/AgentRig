use super::{Job, Record, identity::Identity, storage};
use crate::shared::fs::{self, OpenOptions};
use anyhow::{Result, ensure};
use std::{
    os::unix::process::CommandExt,
    path::Path,
    process::{Command, Stdio},
    time::{Duration, Instant},
};

impl Job {
    pub fn launch(mut self, executable: &Path) -> Result<String> {
        self.record.background = true;
        self.save()?;
        let mut child = self.spawn_launcher(executable)?;
        let deadline = Instant::now() + Duration::from_secs(5);
        while Instant::now() < deadline {
            let adopted = self.was_adopted()?;
            if adopted {
                return Ok(self.record.run_id);
            }
            if let Some(status) = child.try_wait()? {
                let adopted = self.was_adopted()?;
                if adopted {
                    return Ok(self.record.run_id);
                }
                self.finish(
                    status.code().unwrap_or(127),
                    Some("background launcher exited before adoption; see launcher.log".into()),
                )?;
                anyhow::bail!(
                    "background launch failed; inspect run {}",
                    self.record.run_id
                );
            }
            std::thread::sleep(Duration::from_millis(20));
        }
        Ok(self.record.run_id)
    }
    fn spawn_launcher(&mut self, executable: &Path) -> Result<std::process::Child> {
        let mut launcher = self.launcher(executable)?;
        let child = match launcher.spawn() {
            Ok(child) => child,
            Err(error) => {
                self.finish(127, Some(error.to_string()))?;
                return Err(error.into());
            }
        };
        crate::shared::artifacts::json::save(
            &self.directory.join("launcher.json"),
            &Identity::read(child.id()).ok(),
        )?;
        Ok(child)
    }
    fn was_adopted(&self) -> Result<bool> {
        let record: Record =
            serde_json::from_slice(&fs::read(self.directory.join("record.json"))?)?;
        Ok(record.group.is_some())
    }
    pub fn adopt(runtime: &Path, id: &str) -> Result<Self> {
        let mut record = storage::load(runtime, id)?;
        ensure!(record.finished.is_none(), "run already finished");
        ensure!(record.background, "run is not a background launch");
        ensure!(record.group.is_none(), "run already adopted");
        let supervisor = Identity::read(std::process::id())?;
        ensure!(
            super::group::current(&supervisor),
            "runner does not lead its own process group"
        );
        record.group = Some(supervisor.clone());
        record.supervisor = supervisor;
        let job = Self {
            directory: runtime.join("jobs").join(id),
            record,
            started: Instant::now(),
        };
        job.save()?;
        ensure!(
            !job.directory.join("stop-requested").exists(),
            "run cancelled before execution"
        );
        Ok(job)
    }
    pub fn record(&self) -> &Record {
        &self.record
    }
    fn launcher(&self, executable: &Path) -> Result<Command> {
        let log = fs::open_with(
            OpenOptions::new().create_new(true).write(true),
            self.directory.join("launcher.log"),
        )?;
        let mut command = Command::new(executable);
        command
            .arg("_job-run")
            .arg("--root")
            .arg(&self.record.project)
            .arg(&self.record.run_id)
            .stdin(Stdio::null())
            .stdout(log.try_clone()?)
            .stderr(log)
            .process_group(0);
        Ok(command)
    }
}

pub(super) fn launcher_live(directory: &Path) -> Result<bool> {
    let bytes = match fs::read(directory.join("launcher.json")) {
        Ok(bytes) => bytes,
        Err(error) if error.missing() => return Ok(false),
        Err(error) => return Err(error.into()),
    };
    let identity: Option<Identity> = serde_json::from_slice(&bytes)?;
    Ok(identity.is_some_and(|identity| identity.observe().is_some()))
}
