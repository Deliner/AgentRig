pub mod check;
use crate::project_config::{Command, Context};
use crate::shared::paths::patterns::globs;
use crate::structural_lint::architecture::describe;
use anyhow::Result;
use std::{fs, io::Read, os::unix::fs::PermissionsExt, path::Path};

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
    let shebang = fs::File::open(path)
        .and_then(|mut file| file.read_exact(&mut start))
        .is_ok_and(|()| &start == b"#!");
    let mode = fs::metadata(path).is_ok_and(|metadata| metadata.permissions().mode() & 0o111 != 0);
    regular && (shebang || mode)
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
        println!("{name}: {}", command.purpose);
        println!("  just run {name}{}", parameters(command));
        match location(command, &tools) {
            Some(path) => println!("  {}", role(context, path)),
            None => println!("  runs {}", command.argv.join(" ")),
        }
        if let Some(doc) = &command.doc {
            println!("  doc: {doc}");
        }
    }
    let entries = globs(&context.config.tools.entries)?;
    for path in tools.iter().filter(|path| entries.is_match(path)) {
        println!("{}", role(context, path));
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
    let file = Path::new(path);
    let owner = file.parent().map(Path::display);
    match (describe(&context.root, file), owner) {
        (Some((purpose, role)), Some(owner)) => format!("{path}: {role}; owner {owner}: {purpose}"),
        _ => format!("{path}: not described by its architecture.yaml"),
    }
}
fn removed(context: &Context) {
    let removed = &context.config.tools.removed;
    let listed = !removed.is_empty();
    if listed {
        println!("Removed:");
    }
    for (name, item) in removed {
        let restore = item.revision.as_ref().map_or_else(
            || "never committed; not recoverable".to_owned(),
            |revision| format!("restore: git show {revision}:{}", item.path),
        );
        println!("{name}: {}", item.purpose);
        println!("  {}: {}; {restore}", item.path, item.reason);
    }
}
