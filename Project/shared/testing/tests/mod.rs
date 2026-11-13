use serde_yaml_ng::Value;
use std::{
    collections::BTreeMap,
    env,
    ffi::{OsStr, OsString},
    fs,
    io::Write,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    sync::OnceLock,
};
use tempfile::TempDir;

mod launchers;

const FOREIGN: [&str; 8] = [
    "WORKER_OWNER",
    "WORKER_PARENT_RUN",
    "AGENTRIG_SCRATCH",
    "CODEX_THREAD_ID",
    "CODEX_SESSION_ID",
    "CARGO_TARGET_DIR",
    "CARGO_ENCODED_RUSTFLAGS",
    "RUSTFLAGS",
];
const INTERPRETER: &str = "import os, sys; print(os.path.dirname(sys.executable))";
pub const REPAIR: &str = "guides/repair/SKILL.md";
pub const FAIL: &str = "fail:\n  argv: [sh, -c, exit 23]\n";
pub const GATE: &str = "- id: lint
  kind: lint
  skill: guides/repair/SKILL.md
- id: tests
  kind: command
  command: fail
  skill: guides/repair/SKILL.md
  warning: true
";
const SKILL: &str = "---\nname: repair\ndescription: Fix the reported failing check.\n---\n";
const LINT: &str = "version: 1
config_skill: guides/repair/SKILL.md
exclude: [.git/**, .runtime/**]
rules:
- id: lines
  kind: nonblank-lines
  target: file
  include: [src/**]
  warning: 30
  error: 60
  warning_skill: guides/repair/SKILL.md
  error_skill: guides/repair/SKILL.md
";

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

pub fn repository() -> &'static Path {
    static ROOT: OnceLock<PathBuf> = OnceLock::new();
    ROOT.get_or_init(|| {
        let manifest =
            env::var_os("CARGO_MANIFEST_DIR").expect("cargo test sets CARGO_MANIFEST_DIR");
        Path::new(&manifest).parent().unwrap().to_owned()
    })
}

pub fn python_path() -> &'static OsStr {
    static PATH: OnceLock<OsString> = OnceLock::new();
    PATH.get_or_init(|| {
        let tooling = repository().join("tooling");
        let output = Command::new("uv")
            .args(["run", "--locked", "--project"])
            .arg(tooling)
            .args(["python", "-I", "-c", INTERPRETER])
            .current_dir(repository())
            .output()
            .unwrap();
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(output.status.success(), "tooling Python: {stderr}");
        let bin = PathBuf::from(String::from_utf8(output.stdout).unwrap().trim());
        search_path(bin)
    })
}

pub fn search_path(first: PathBuf) -> OsString {
    let inherited = std::env::var_os("PATH").unwrap_or_default();
    let paths = std::iter::once(first).chain(std::env::split_paths(&inherited));
    std::env::join_paths(paths).unwrap()
}

// DECISION: D160
fn build() -> PathBuf {
    let root = repository();
    let status = Command::new(root.join("tooling/worker/build"))
        .arg("--version")
        .current_dir(root)
        .stdout(Stdio::null())
        .status()
        .unwrap();
    assert!(status.success(), "the candidate runtime build failed");
    let target = std::env::var_os("CARGO_TARGET_DIR")
        .map_or_else(|| root.join(".cache/worker"), PathBuf::from);
    target.join("release/agentrig")
}

pub fn hermetic(program: impl AsRef<OsStr>) -> Command {
    let mut command = Command::new(program);
    for (key, _) in std::env::vars_os() {
        let name = key.to_string_lossy();
        let foreign = name.starts_with("GIT_") || FOREIGN.contains(&name.as_ref());
        if foreign {
            command.env_remove(&key);
        }
    }
    command
}

pub fn runtime(root: &Path, args: &[&str]) -> Command {
    launch(worker(), root, args)
}

pub fn launch(program: &Path, root: &Path, args: &[&str]) -> Command {
    let (name, rest) = args.split_first().unwrap();
    let mut command = hermetic(program);
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

pub fn project(root: &Path, commands: &str, checks: &str) {
    let version = env!("CARGO_PKG_VERSION");
    let identity = format!(
        "version: 1\nruntime: \"{version}\"\nconfig_skill: {REPAIR}\npaths:\n  sources: [src/**]\n  memory: notes\n  skills: guides\n  lint: lint.yaml\n  runtime: .runtime\n"
    );
    let files = [
        (REPAIR, SKILL),
        (".agentrig/project.yaml", &identity),
        (".agentrig/git.yaml", "base: trunk\nprefix: task/\n"),
        (".agentrig/commands.yaml", commands),
        (".agentrig/checks.yaml", checks),
        ("lint.yaml", LINT),
    ];
    for (name, text) in files {
        let path = root.join(name);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, text).unwrap();
    }
    fs::create_dir_all(root.join("src")).unwrap();
}

pub fn memory(root: &Path) {
    let notes = root.join("notes");
    let indexes = [
        (
            "Plan",
            "ID | Status | Depends on | Feature | User capability",
        ),
        ("Decisions", "ID | Decision | Applies in"),
        ("Invariants", "ID | Invariant | Enforced by"),
    ];
    for (name, headers) in indexes {
        fs::create_dir_all(notes.join(name)).unwrap();
        let index = format!("# {name}\n\n| {headers} |\n");
        fs::write(notes.join(format!("{name}.md")), index).unwrap();
    }
    let sections = [
        "Focus",
        "Workspace",
        "Progress",
        "Verification",
        "Blockers",
        "Next action",
    ];
    let sections = sections.map(|section| format!("## {section}\n\nNone."));
    let state = format!("# State\n\n{}", sections.join("\n\n"));
    fs::write(notes.join("State.md"), state).unwrap();
}

pub fn checked_repository() -> (TempDir, PathBuf) {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().canonicalize().unwrap();
    project(
        &root,
        FAIL,
        &GATE.replace("warning: true", "warning: false"),
    );
    memory(&root);
    fs::write(root.join(".gitignore"), ".runtime/\n").unwrap();
    fs::write(root.join("src/value.py"), "value = 1\n").unwrap();
    git(&root, &["init", "-q", "-b", "trunk"]);
    git(&root, &["config", "user.name", "Test"]);
    git(&root, &["config", "user.email", "test@example.invalid"]);
    git(&root, &["add", "."]);
    git(&root, &["commit", "-qm", "baseline"]);
    (temp, root)
}

#[derive(Debug, PartialEq)]
pub enum Entry {
    Directory,
    File(Vec<u8>),
}

pub type Snapshot = BTreeMap<PathBuf, Entry>;

fn entries(directory: &Path, found: &mut Snapshot) {
    for entry in fs::read_dir(directory).unwrap() {
        let path = entry.unwrap().path();
        let nested = path.is_dir();
        if nested {
            entries(&path, found);
            found.insert(path, Entry::Directory);
        } else {
            let contents = fs::read(&path).unwrap();
            found.insert(path, Entry::File(contents));
        }
    }
}

pub fn snapshot(root: &Path) -> Snapshot {
    let mut found = BTreeMap::new();
    entries(root, &mut found);
    found
}

pub fn rerun(output: &str) -> Run {
    let (_, guidance) = output.split_once("RERUN: ").unwrap();
    replay(guidance.lines().next().unwrap())
}

pub fn replay(command: &str) -> Run {
    let argv = shell_words::split(command).unwrap();
    let (program, args) = argv.split_first().unwrap();
    run(
        hermetic(program).args(args).current_dir(env::temp_dir()),
        "",
    )
}
