// DECISION: D022
use std::{fmt, path::Path};

pub struct Guidance<'a> {
    pub level: &'a str,
    pub id: &'a str,
    pub location: &'a str,
    pub message: &'a str,
    pub skill: &'a str,
    pub rerun: &'a str,
}
impl fmt::Display for Guidance<'_> {
    fn fmt(&self, output: &mut fmt::Formatter<'_>) -> fmt::Result {
        let advisory = self.level.eq_ignore_ascii_case("warning");
        let action = if advisory { "Consider" } else { "Apply" };
        write!(
            output,
            "{} [{}] {}: {}. ACTION: {action} {}. RERUN: {}",
            self.level.to_uppercase(),
            self.id,
            self.location,
            self.message,
            self.skill,
            self.rerun
        )
    }
}
pub fn rerun(root: &Path, args: &[String]) -> String {
    let binary = executable();
    let mut argv = vec![binary.to_string_lossy().into_owned()];
    argv.extend(args.first().cloned());
    argv.extend(["--root".into(), root.to_string_lossy().into_owned()]);
    argv.extend(args.iter().skip(1).cloned());
    shell_words::join(argv)
}

pub fn failure(error: &anyhow::Error) -> String {
    let (root, args) = invocation();
    let command = args.first().map(String::as_str).unwrap_or("worker");
    let config =
        review_runner::config::yaml::read::<serde_json::Value>(&root.join("agentrig.yaml")).ok();
    let skill = repair_skill(config.as_ref(), command);
    let binary = executable();
    let rerun = shell_words::join(
        std::iter::once(binary.to_string_lossy().into_owned()).chain(args.iter().cloned()),
    );
    Guidance {
        level: "ERROR",
        id: command,
        location: &root.display().to_string(),
        message: &format!("{error:#}"),
        skill,
        rerun: &rerun,
    }
    .to_string()
}

fn invocation() -> (std::path::PathBuf, Vec<String>) {
    let mut args: Vec<String> = std::env::args().skip(1).collect();
    let option = args
        .iter()
        .take_while(|arg| arg.as_str() != "--")
        .position(|arg| arg == "--root");
    let value = option.and_then(|index| args.get(index + 1));
    let cwd = std::env::current_dir().unwrap_or_default();
    let root = value.map(|value| cwd.join(value)).unwrap_or(cwd);
    let root = root.canonicalize().unwrap_or(root);
    match option {
        Some(index) => {
            if let Some(value) = args.get_mut(index + 1) {
                *value = root.to_string_lossy().into_owned();
            }
        }
        None => {
            let position = args.len().min(1);
            args.splice(
                position..position,
                ["--root".into(), root.to_string_lossy().into_owned()],
            );
        }
    }
    (root, args)
}
fn repair_skill<'a>(config: Option<&'a serde_json::Value>, command: &str) -> &'a str {
    let memory = command == "memory-check";
    let check_skill = config
        .and_then(|config| config.get("checks"))
        .and_then(serde_json::Value::as_array)
        .and_then(|checks| {
            checks.iter().find(|check| {
                memory && check.get("kind").and_then(serde_json::Value::as_str) == Some("memory")
            })
        })
        .and_then(|check| check.get("skill"));
    check_skill
        .or_else(|| config.and_then(|config| config.get("config_skill")))
        .and_then(serde_json::Value::as_str)
        .unwrap_or("config_skill in agentrig.yaml (repair configuration syntax first)")
}

fn executable() -> std::path::PathBuf {
    let observed = std::env::current_exe().unwrap_or_else(|_| "agentrig".into());
    let available = observed.is_file();
    if available {
        return observed;
    }
    // Linux marks a replaced running executable as deleted; rerun its installed successor.
    observed
        .file_name()
        .and_then(|name| name.to_str())
        .and_then(|name| name.strip_suffix(" (deleted)"))
        .map(|name| observed.with_file_name(name))
        .filter(|path| path.is_file())
        .unwrap_or(observed)
}
