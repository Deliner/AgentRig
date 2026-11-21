use crate::project_config::{Command, Context};
use crate::shared::fs_with_context::{self as fs, PermissionsExt};
use crate::shared::paths::globs;
use anyhow::Result;
use std::{io::Read, path::Path};

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
