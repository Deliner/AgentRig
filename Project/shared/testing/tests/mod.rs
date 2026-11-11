use serde_yaml_ng::Value;
use std::{
    ffi::OsStr,
    fs,
    io::Write,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    sync::OnceLock,
};

const IDENTITY: [&str; 5] = [
    "WORKER_OWNER",
    "WORKER_PARENT_RUN",
    "AGENTRIG_SCRATCH",
    "CODEX_THREAD_ID",
    "CODEX_SESSION_ID",
];

pub struct Run {
    pub code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
}

impl Run {
    pub fn output(&self) -> String {
        format!("{}{}", self.stdout, self.stderr)
    }
}

pub fn worker() -> &'static Path {
    static WORKER: OnceLock<PathBuf> = OnceLock::new();
    WORKER.get_or_init(|| {
        let configured = std::env::var_os("WORKER_BINARY").map(PathBuf::from);
        let binary = configured.unwrap_or_else(build);
        binary
            .canonicalize()
            .unwrap_or_else(|error| panic!("{}: {error}", binary.display()))
    })
}

fn build() -> PathBuf {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
    let status = Command::new(root.join("tooling/worker/build"))
        .arg("--version")
        .current_dir(root)
        .stdout(Stdio::null())
        .status()
        .unwrap();
    assert!(status.success(), "the candidate runtime build failed");
    let target = std::env::var_os("WORKER_TARGET_DIR")
        .map_or_else(|| root.join(".cache/worker"), PathBuf::from);
    target.join("release/agentrig")
}

pub fn hermetic(program: impl AsRef<OsStr>) -> Command {
    let mut command = Command::new(program);
    for (key, _) in std::env::vars_os() {
        let name = key.to_string_lossy();
        let foreign = name.starts_with("GIT_") || IDENTITY.contains(&name.as_ref());
        if foreign {
            command.env_remove(&key);
        }
    }
    command
}

pub fn runtime(root: &Path, args: &[&str]) -> Command {
    let (name, rest) = args.split_first().unwrap();
    let mut command = hermetic(worker());
    command
        .arg(name)
        .arg("--root")
        .arg(root)
        .args(rest)
        .current_dir(root);
    command
}

pub fn run(command: &mut Command, input: &str) -> Run {
    let mut child = command
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let mut stdin = child.stdin.take().unwrap();
    stdin.write_all(input.as_bytes()).unwrap();
    drop(stdin);
    let output = child.wait_with_output().unwrap();
    Run {
        code: output.status.code(),
        stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
    }
}

pub fn invoke(root: &Path, args: &[&str]) -> Run {
    run(&mut runtime(root, args), "")
}

pub fn git_run(root: &Path, args: &[&str]) -> Run {
    run(hermetic("git").args(args).current_dir(root), "")
}

pub fn git(root: &Path, args: &[&str]) -> String {
    let result = git_run(root, args);
    assert_eq!(result.code, Some(0), "git {args:?}: {}", result.output());
    result.stdout.trim().to_owned()
}

pub fn commit(root: &Path, name: &str, text: &str) {
    fs::write(root.join(name), text).unwrap();
    git(root, &["add", "."]);
    git(root, &["commit", "-qm", name]);
}

pub fn configure(root: &Path, module: &str, addition: &str) {
    let path = root.join(".agentrig").join(format!("{module}.yaml"));
    let mut current: Value = serde_yaml_ng::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
    let addition: Value = serde_yaml_ng::from_str(addition).unwrap();
    if let (Some(entries), Some(added)) = (current.as_mapping_mut(), addition.as_mapping()) {
        entries.extend(added.clone());
    } else if let (Some(entries), Some(added)) = (current.as_sequence_mut(), addition.as_sequence())
    {
        entries.extend(added.iter().cloned());
    } else {
        panic!("{module}: the addition must have the configured shape");
    }
    fs::write(path, serde_yaml_ng::to_string(&current).unwrap()).unwrap();
}
