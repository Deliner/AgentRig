use std::path::Path;

pub fn rerun(root: &Path, args: &[String]) -> String {
    let binary = executable();
    let mut argv = vec![binary.to_string_lossy().into_owned()];
    argv.extend(args.first().cloned());
    argv.extend(["--root".into(), root.to_string_lossy().into_owned()]);
    argv.extend(args.iter().skip(1).cloned());
    shell_words::join(argv)
}

pub(super) struct Invocation {
    pub(super) root: std::path::PathBuf,
    pub(super) args: Vec<String>,
}
pub(super) fn invocation() -> Invocation {
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
    Invocation { root, args }
}

pub(super) fn executable() -> std::path::PathBuf {
    let observed = std::env::current_exe().unwrap_or_else(|_| "agentrig".into());
    let available = observed.is_file();
    if available {
        return observed;
    }
    let installed_successor = observed
        .file_name()
        .and_then(|name| name.to_str())
        .and_then(|name| name.strip_suffix(" (deleted)"))
        .map(|name| observed.with_file_name(name))
        .filter(|path| path.is_file());
    installed_successor.unwrap_or(observed)
}
