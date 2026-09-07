use super::report;
use crate::delegate::code::Workspace;
use crate::{
    delegate::{
        config,
        sandbox::{self, Layout},
        task::{self, Request},
    },
    jobs,
};
use anyhow::{Result, ensure};
use review_runner::artifacts::json::save as save_json;
use serde_json::{Value, json};
use std::{fs, path::Path};
use std::{
    io::Write,
    process::{Command, Output},
    time::{Duration, Instant},
};

pub(super) fn run(directory: &Path) -> Result<Value> {
    let profile: config::Profile =
        serde_json::from_slice(&fs::read(directory.join("profile.json"))?)?;
    let request: Request = serde_json::from_slice(&fs::read(directory.join("request.json"))?)?;
    let layout = Layout {
        input: directory.join("input"),
        private: directory.join("private"),
        codex: serde_json::from_slice(&fs::read(directory.join("codex.json"))?)?,
    };
    let deadline = Instant::now() + Duration::from_secs(profile.timeout_seconds);
    let structured = matches!(profile.frontend, config::Frontend::ClaudeCode);
    let output = invoke(sandbox::command(&layout, &profile)?, deadline, structured)?;
    if structured {
        std::io::stdout().write_all(&output.stdout)?;
        std::io::stderr().write_all(&output.stderr)?;
    }
    ensure!(
        output.status.success(),
        "executor exited {} (124 indicates timeout)",
        jobs::process::exit_code(&output)
    );
    sandbox::response(&layout, &profile, &output.stdout)?;
    if let Some(changes) = &request.contract.changes {
        let mut code = patch(directory, changes, &profile)?;
        let results = checks(&layout, &profile, changes, deadline)?;
        let passed = results.values().all(|value| value["exit_code"] == 0);
        code["checks"] = json!(results);
        code["verified"] = json!(passed);
        save_json(&directory.join("code-report.json"), &code)?;
        ensure!(passed, "code checks failed; see code-report.json");
    }
    let verified = task::verify(&layout.private.join("work"), &request.contract)?;
    report::artifacts(directory, &request.contract)?;
    Ok(verified)
}

fn patch(directory: &Path, changes: &task::Changes, profile: &config::Profile) -> Result<Value> {
    let workspace = Workspace::open(&directory.join("private/code"));
    let (patch, mut report) = workspace.patch(&changes.write_paths)?;
    let visible = review_runner::config::globs(&profile.visible_paths)?;
    for path in report["changed_paths"].as_array().unwrap() {
        ensure!(
            visible.is_match(path.as_str().unwrap()),
            "code change outside visible_paths: {path}"
        );
    }
    let inputs: task::Inputs = serde_json::from_slice(&fs::read(directory.join("inputs.json"))?)?;
    report["base"] = json!(inputs.revision);
    report["vcs"] = json!(inputs.vcs);
    report["patch"] = json!("change.patch");
    report["checks"] = json!({});
    report["verified"] = json!(false);
    fs::write(directory.join("change.patch"), patch)?;
    save_json(&directory.join("code-report.json"), &report)?;
    Ok(report)
}

fn checks(
    layout: &Layout,
    profile: &config::Profile,
    changes: &task::Changes,
    deadline: Instant,
) -> Result<std::collections::BTreeMap<String, Value>> {
    let mut results = std::collections::BTreeMap::new();
    for (name, argv) in &changes.checks {
        let result = match sandbox::check(layout, profile, argv)
            .and_then(|command| invoke(command, deadline, true))
        {
            Ok(output) => json!({"argv":argv,"exit_code":jobs::process::exit_code(&output),
                "stdout":String::from_utf8_lossy(&output.stdout),"stderr":String::from_utf8_lossy(&output.stderr)}),
            Err(error) => json!({"argv":argv,"error":format!("{error:#}"),"exit_code":null}),
        };
        results.insert(name.clone(), result);
    }
    Ok(results)
}

fn invoke(sandbox: Command, deadline: Instant, capture: bool) -> Result<Output> {
    let remaining = deadline.saturating_duration_since(Instant::now());
    ensure!(
        !remaining.is_zero(),
        "delegate execution deadline exhausted"
    );
    let mut command = Command::new("timeout");
    command
        .args(["--kill-after=2s", &remaining.as_secs_f64().to_string()])
        .arg(sandbox.get_program())
        .args(sandbox.get_args());
    command.env_clear().envs(
        sandbox
            .get_envs()
            .filter_map(|(key, value)| value.map(|value| (key, value))),
    );
    jobs::process::execute(&mut command, capture, None)
}
