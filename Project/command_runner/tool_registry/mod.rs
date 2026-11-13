pub mod placement_check;
mod removal_records;
#[cfg(test)]
mod tests;
use crate::project_config::{Command, Context};
use crate::shared::console;
use crate::shared::fs_with_context::{self as fs, PermissionsExt};
use crate::shared::paths::glob_patterns::globs;
use crate::structural_lint::architecture::Rooted;
use anyhow::Result;
use std::{io::Read, path::Path};

struct Finding {
    path: String,
    message: String,
}
type Findings = Vec<Finding>;

pub fn tools(context: &Context, files: &[String]) -> Result<Vec<String>> {
    let recognized = globs(&context.config.tools.recognize)?;
    Ok(files
        .iter()
        .filter(|path| recognized.is_match(path) || executable(&context.root.join(path)))
        .cloned()
        .collect())
}
fn executable(path: &Path) -> bool {
    let regular = fs::symlink_metadata(path).is_ok_and(|metadata| metadata.is_file());
    let mut start = [0; 2];
    let shebang =
        fs::open(path).is_ok_and(|mut file| file.read_exact(&mut start).is_ok() && &start == b"#!");
    let mode = fs::metadata(path).is_ok_and(|metadata| metadata.permissions().mode() & 0o111 != 0);
    let runnable = shebang || mode;
    regular && runnable
}
pub fn location<'a>(command: &'a Command, tools: &[String]) -> Option<&'a String> {
    command
        .argv
        .iter()
        .find(|argument| tools.contains(argument))
}
pub fn list(context: &Context) -> Result<()> {
    let files = crate::shared::vcs::Repository::new(&context.root).working_files()?;
    let tools = tools(context, &files)?;
    for (name, command) in &context.config.commands {
        console::stdout(format!("{name}: {}", command.purpose));
        console::stdout(format!("  just run {name}{}", parameters(command)));
        let fixed = !command.argv.is_empty();
        match (location(command, &tools), fixed) {
            (Some(path), _) => console::stdout(format!("  {}", role(context, path))),
            (None, true) => console::stdout(format!("  runs {}", command.argv.join(" "))),
            (None, false) => {}
        }
        if let Some(doc) = &command.doc {
            console::stdout(format!("  doc: {doc}"));
        }
    }
    let entries = globs(&context.config.tools.entries)?;
    for path in tools.iter().filter(|path| entries.is_match(path)) {
        console::stdout(role(context, path));
    }
    removed(context);
    Ok(())
}
fn parameters(command: &Command) -> String {
    let programs: Vec<_> = command.programs.keys().cloned().collect();
    match (programs.is_empty(), command.accepts_args) {
        (false, _) => format!(" -- PROGRAM ARGS; programs: {}", programs.join(", ")),
        (true, true) => " -- ARGS".into(),
        (true, false) => String::new(),
    }
}
fn role(context: &Context, path: &str) -> String {
    let root = &context.root;
    let file = Path::new(path);
    let owner = file.parent().map(Path::display);
    let rooted = Rooted { root, path: file };
    match (rooted.describe(), owner) {
        (Some(description), Some(owner)) => format!(
            "{path}: {role}; owner {owner}: {purpose}",
            role = description.role,
            purpose = description.purpose
        ),
        _ => format!("{path}: not described by its architecture.yaml"),
    }
}
fn removed(context: &Context) {
    let removed = &context.config.tools.removed;
    let listed = !removed.is_empty();
    if listed {
        console::stdout("Removed:");
    }
    for (name, item) in removed {
        let restore = item.revision.as_ref().map_or_else(
            || "never committed; not recoverable".to_owned(),
            |revision| format!("restore: git show {revision}:{}", item.path),
        );
        console::stdout(format!("{name}: {}", item.purpose));
        console::stdout(format!(
            "  {path}: {reason}; {restore}",
            path = item.path,
            reason = item.reason
        ));
    }
}
