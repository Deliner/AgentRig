use super::{Job, Record, identity::Identity, scope::Scope, storage};
use anyhow::{Result, ensure};
use std::{
    fs::{self, OpenOptions},
    os::unix::process::CommandExt,
    path::Path,
    process::{Command, Stdio},
    time::{Duration, Instant},
};

impl Job {
    pub fn launch(mut self, executable: &Path) -> Result<String> {
        self.record.background = true;
        let scope = Scope::new(&self.record.run_id);
        self.record.scope = Some(scope.clone());
        self.save()?;
        let mut launcher = self.launcher(executable, &scope)?;
        let mut child = match launcher.spawn() {
            Ok(child) => child,
            Err(error) => {
                self.finish(127, Some(error.to_string()))?;
                return Err(error.into());
            }
        };
        let deadline = Instant::now() + Duration::from_secs(5);
        while Instant::now() < deadline {
            let record: Record =
                serde_json::from_slice(&fs::read(self.directory.join("record.json"))?)?;
            let adopted = record
                .scope
                .as_ref()
                .is_some_and(|scope| scope.invocation.is_some());
            if adopted {
                return Ok(self.record.run_id);
            }
            if let Some(status) = child.try_wait()? {
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
        // A slow observation does not authorize starting a second process.
        Ok(self.record.run_id)
    }
    pub fn adopt(runtime: &Path, id: &str) -> Result<Self> {
        let mut record = storage::load(runtime, id)?;
        ensure!(record.finished.is_none(), "run already finished");
        let scope = record
            .scope
            .as_mut()
            .ok_or_else(|| anyhow::anyhow!("run is not a scoped launch"))?;
        ensure!(scope.invocation.is_none(), "run already adopted");
        let actual = fs::read_to_string("/proc/self/cgroup")?;
        ensure!(
            actual.trim().ends_with(&format!("/{}", scope.unit)),
            "runner is outside its assigned scope"
        );
        scope.attach()?;
        record.supervisor = Identity::read(std::process::id())?;
        let job = Self {
            directory: runtime.join("jobs").join(id),
            record,
            started: Instant::now(),
        };
        job.save()?;
        Ok(job)
    }
    pub fn record(&self) -> &Record {
        &self.record
    }
    pub fn foreground(mut self, executable: &Path, capture: bool) -> Result<std::process::Output> {
        let scope = Scope::new(&self.record.run_id);
        self.record.scope = Some(scope.clone());
        self.save()?;
        let mut command = self.launcher(executable, &scope)?;
        let result = super::process::execute(&mut command, capture, None);
        let saved: Record = serde_json::from_slice(&fs::read(self.directory.join("record.json"))?)?;
        let adopted = saved
            .scope
            .as_ref()
            .is_some_and(|scope| scope.invocation.is_some());
        let missing_runner = !adopted;
        if missing_runner {
            self.finish(127, Some("scope launcher failed before adoption".into()))?;
            anyhow::bail!(
                "scope launcher failed before adoption for run {}",
                self.record.run_id
            );
        }
        result
    }
    fn launcher(&self, executable: &Path, scope: &Scope) -> Result<Command> {
        let mut command = Command::new("systemd-run");
        command
            .args([
                "--user",
                "--scope",
                "--quiet",
                "--no-ask-password",
                "--collect",
                "--property=TimeoutStopSec=2s",
                "--unit",
            ])
            .arg(&scope.unit)
            .arg("--")
            .arg(executable)
            .arg("_job-run")
            .arg("--root")
            .arg(&self.record.project)
            .arg(&self.record.run_id);
        if self.record.background {
            let log = OpenOptions::new()
                .create_new(true)
                .write(true)
                .open(self.directory.join("launcher.log"))?;
            command
                .stdin(Stdio::null())
                .stdout(log.try_clone()?)
                .stderr(log)
                .process_group(0);
        }
        Ok(command)
    }
}
