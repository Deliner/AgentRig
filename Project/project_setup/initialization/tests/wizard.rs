use super::{names, text, yaml};
use crate::project_setup::initialization::init;
use crate::shared::testing::{Run, hermetic, launch, run, snapshot, worker};
use std::{
    collections::BTreeSet,
    fs,
    io::{Read, Write},
    path::Path,
    process::{ChildStdout, Command, Stdio},
    sync::mpsc,
    thread::{self, JoinHandle},
    time::Duration,
};

const QUESTIONS: usize = 11;
const CONFIRMATION: &str = "Install this environment?";
const CHANGE: &str = "User change after preview.\n";
const COMPARED: [&str; 11] = [
    ".agentrig/project.yaml",
    ".agentrig/git.yaml",
    ".agentrig/commands.yaml",
    ".agentrig/checks.yaml",
    ".agentrig/hooks.yaml",
    ".agentrig/oracles.yaml",
    ".agentrig/tools.yaml",
    ".agentrig/rust.yaml",
    "AGENTS.md",
    "justfile",
    ".codex/config.toml",
];

fn wizard(base: &Path, root: &Path) -> Command {
    let mut command = hermetic(worker());
    command
        .args(["init", "--interactive", "--root"])
        .arg(root)
        .current_dir(base);
    command
}

fn interactive(base: &Path, root: &Path, answers: &[&str]) -> Run {
    let input: String = answers.iter().map(|answer| format!("{answer}\n")).collect();
    run(&mut wizard(base, root), &input)
}

fn installed(launcher: &Path, root: &Path) {
    let result = run(&mut launch(launcher, root, &["config-check"]), "");
    assert_eq!(result.code, Some(0), "{}", result.output());
}

fn matches_declarative(language: &str) {
    let temp = tempfile::tempdir().unwrap();
    let base = temp.path().canonicalize().unwrap();
    let target = base.join("interactive");
    let mut answers = [""; QUESTIONS];
    answers[1] = language;
    answers[QUESTIONS - 1] = "yes";
    let result = interactive(&base, &target, &answers);
    assert_eq!(result.code, Some(0), "{}", result.output());
    assert!(
        result.stdout.contains("Configuration for"),
        "{}",
        result.stdout
    );
    let ordinary = base.join("ordinary");
    fs::create_dir(&ordinary).unwrap();
    let declared = ["--language".to_owned(), language.to_owned()];
    assert_eq!(init(&ordinary, &declared).unwrap(), 0);
    for name in COMPARED {
        let delivered = fs::read(target.join(name)).unwrap();
        assert_eq!(delivered, fs::read(ordinary.join(name)).unwrap(), "{name}");
    }
    installed(&target.join(".agentrig/bin/agentrig"), &target);
}

fn preview(mut stdout: ChildStdout) -> (mpsc::Receiver<()>, JoinHandle<String>) {
    let (sender, shown) = mpsc::channel();
    let reader = thread::spawn(move || {
        let mut seen = Vec::new();
        let mut chunk = [0; 4096];
        loop {
            let read = stdout.read(&mut chunk).unwrap();
            seen.extend_from_slice(&chunk[..read]);
            let asked = String::from_utf8_lossy(&seen).contains(CONFIRMATION);
            if asked {
                break;
            }
            assert!(read > 0, "{}", String::from_utf8_lossy(&seen));
        }
        sender.send(()).unwrap();
        stdout.read_to_end(&mut seen).unwrap();
        String::from_utf8_lossy(&seen).into_owned()
    });
    (shown, reader)
}

#[test]
fn interactive_python_init_matches_declarative_init() {
    matches_declarative("python");
}

#[test]
fn interactive_rust_init_matches_declarative_init() {
    matches_declarative("rust");
}

#[test]
fn interactive_cancellation_leaves_no_target() {
    let skipped = [""; QUESTIONS - 1];
    let declined: Vec<&str> = skipped.iter().copied().chain(["no"]).collect();
    let cases: [&[&str]; 4] = [&[], &["cancel"], &skipped, &declined];
    for answers in cases {
        let temp = tempfile::tempdir().unwrap();
        let base = temp.path().canonicalize().unwrap();
        let result = interactive(&base, &base.join("missing/consumer"), answers);
        assert_eq!(result.code, Some(0), "{answers:?}: {}", result.output());
        assert!(
            result.stdout.contains("cancelled"),
            "{answers:?}: {}",
            result.stdout
        );
        assert!(!base.join("missing").exists(), "{answers:?}");
    }
}

#[test]
fn interactive_invalid_selection_preserves_existing_files() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().canonicalize().unwrap();
    fs::write(root.join("user.txt"), "Keep user content.\n").unwrap();
    let before = snapshot(&root);
    let mut answers = [""; QUESTIONS];
    answers[1] = "unsupported-language";
    let result = interactive(&root, &root, &answers);
    assert_eq!(result.code, Some(2), "{}", result.output());
    assert!(
        result.stderr.contains("python or rust"),
        "{}",
        result.stderr
    );
    assert_eq!(snapshot(&root), before);
}

#[test]
fn interactive_selects_layout_client_and_checks() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().canonicalize().unwrap();
    let mut answers = [""; QUESTIONS];
    answers[2] = "wizard rig";
    answers[4] = "notes";
    answers[8] = "claude-code";
    answers[9] = "lint,memory";
    answers[10] = "yes";
    let result = interactive(&root, &root, &answers);
    assert_eq!(result.code, Some(0), "{}", result.output());
    let paths = &yaml(&root.join(".agentrig/project.yaml"))["paths"];
    assert_eq!(text(&paths["service"]), "wizard rig");
    assert_eq!(text(&paths["skills"]), "wizard rig/skills");
    assert_eq!(text(&paths["memory"]), "notes");
    let checks = yaml(&root.join(".agentrig/checks.yaml"));
    let selected = names(&checks, |check| text(&check["id"]));
    assert_eq!(selected, BTreeSet::from(["lint".into(), "memory".into()]));
    assert!(root.join(".claude/settings.json").is_file());
    assert!(!root.join(".mcp.json").exists());
    installed(&root.join("wizard rig/bin/agentrig"), &root);
}

#[test]
fn interactive_preserves_changes_after_preview() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().canonicalize().unwrap();
    let mut child = wizard(&root, &root)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let mut stdin = child.stdin.take().unwrap();
    stdin
        .write_all("\n".repeat(QUESTIONS - 1).as_bytes())
        .unwrap();
    let (shown, reader) = preview(child.stdout.take().unwrap());
    let waited = shown.recv_timeout(Duration::from_secs(10));
    waited.expect("the wizard shows the confirmation prompt");
    let note = root.join("AGENTS.md");
    fs::write(&note, CHANGE).unwrap();
    stdin.write_all(b"yes\n").unwrap();
    drop(stdin);
    let finished = child.wait_with_output().unwrap();
    let stdout = reader.join().unwrap();
    let stderr = String::from_utf8_lossy(&finished.stderr);
    assert_eq!(finished.status.code(), Some(2), "{stdout}{stderr}");
    assert!(stderr.contains("collision"), "{stderr}");
    assert_eq!(fs::read_to_string(&note).unwrap(), CHANGE);
    assert!(!root.join(".git").exists());
    assert!(!root.join(".agentrig/project.yaml").exists());
}
