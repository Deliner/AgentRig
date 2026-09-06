use anyhow::{Context, Result, ensure};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{collections::BTreeMap, fs, path::Path, process::Command};

#[derive(Clone, Deserialize, Serialize)]
pub struct Scope {
    pub unit: String,
    pub invocation: Option<String>,
}
impl Scope {
    pub fn new(id: &str) -> Self {
        Self {
            unit: format!("worker-{id}.scope"),
            invocation: None,
        }
    }
    pub fn attach(&mut self) -> Result<()> {
        let properties = properties(&self.unit)?;
        let invocation = properties
            .get("InvocationID")
            .filter(|value| !value.is_empty())
            .context("scope has no invocation ID")?;
        self.invocation = Some(invocation.clone());
        Ok(())
    }
    pub fn observe(&self) -> Result<Value> {
        let properties = properties(&self.unit)?;
        let present = properties
            .get("LoadState")
            .is_some_and(|state| state != "not-found");
        let matches = present
            && self
                .invocation
                .as_ref()
                .is_none_or(|id| properties.get("InvocationID") == Some(id));
        let stale = !matches;
        if stale {
            return Ok(json!({"present": false, "populated": false}));
        }
        let group = properties
            .get("ControlGroup")
            .context("scope has no control group")?;
        let empty = group.is_empty();
        if empty {
            return Ok(json!({"present": true, "populated": false}));
        }
        ensure!(
            group.starts_with('/') && !group.split('/').any(|part| part == ".."),
            "invalid control group"
        );
        let path = Path::new("/sys/fs/cgroup").join(&group[1..]);
        let events = match fs::read_to_string(path.join("cgroup.events")) {
            Ok(events) => events,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                return Ok(json!({"present": true, "populated": false}));
            }
            Err(error) => return Err(error.into()),
        };
        Ok(
            json!({"present": true, "populated": events.lines().any(|line| line == "populated 1"),
            "control_group": group, "active_state": properties.get("ActiveState"),
            "memory_bytes": fs::read_to_string(path.join("memory.current")).ok().and_then(|value| value.trim().parse::<u64>().ok()),
            "cpu_stat": fs::read_to_string(path.join("cpu.stat")).ok()}),
        )
    }
    pub fn signal(&self, force: bool) -> Result<()> {
        let properties = properties(&self.unit)?;
        ensure!(
            self.invocation.is_some() && properties.get("InvocationID") == self.invocation.as_ref(),
            "scope identity changed; refusing to stop a replacement"
        );
        let mut command = Command::new("systemctl");
        command.args(["--user", "--no-ask-password"]);
        if force {
            command.args(["kill", "--kill-whom=all", "--signal=KILL"]);
        } else {
            command.args(["stop", "--no-block"]);
        }
        let output = command.arg(&self.unit).output()?;
        ensure!(
            output.status.success(),
            "scope stop failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        Ok(())
    }
}
fn properties(unit: &str) -> Result<BTreeMap<String, String>> {
    let output = Command::new("systemctl")
        .args([
            "--user",
            "--no-ask-password",
            "show",
            unit,
            "--property=LoadState,ActiveState,ControlGroup,InvocationID",
        ])
        .output()?;
    ensure!(
        output.status.success(),
        "cannot observe systemd scope: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    Ok(String::from_utf8(output.stdout)?
        .lines()
        .filter_map(|line| line.split_once('='))
        .map(|(key, value)| (key.into(), value.into()))
        .collect())
}
pub fn capability() -> Value {
    let cgroup_v2 = Path::new("/sys/fs/cgroup/cgroup.controllers").is_file();
    let probe = Command::new("systemd-run")
        .args([
            "--user",
            "--scope",
            "--quiet",
            "--no-ask-password",
            "--collect",
            "--property=TimeoutStopSec=2s",
            "--",
            "true",
        ])
        .output();
    match probe {
        Ok(output) => {
            json!({"backend": "systemd user scope", "available": cgroup_v2 && output.status.success(),
            "cgroup_v2": cgroup_v2, "diagnostic": String::from_utf8_lossy(&output.stderr).trim()})
        }
        Err(error) => {
            json!({"backend": "systemd user scope", "available": false, "cgroup_v2": cgroup_v2, "diagnostic": error.to_string()})
        }
    }
}
