use super::installed_runtime;
use crate::shared::testing::{
    Run, configure, git, git_run, hermetic, invoke, launch, python_path, repository, run, worker,
};
use serde_json::{Value, json};
use std::{
    ffi::OsStr,
    fs,
    path::{Path, PathBuf},
};
use tempfile::TempDir;

const IGNORED: &str = "**/target/\n**/__pycache__/\n**/.pytest_cache/\n";
const GENERATED: [&str; 3] = ["target", "__pycache__", ".pytest_cache"];
const DETAIL: &str =
    "# I001\n\n## Predicate\n\nDoubles correctly.\n\n## Oracle\n\nRun configured I001.\n";

struct Layout {
    language: &'static str,
    base: &'static str,
    prefix: &'static str,
    source: &'static str,
    memory: &'static str,
    skills: &'static str,
    runner: &'static str,
    target: &'static str,
    test: &'static str,
    function: &'static str,
}

const PYTHON: Layout = Layout {
    language: "python",
    base: "trunk",
    prefix: "topic/",
    source: "application",
    memory: "notes",
    skills: "guides",
    runner: "pytest",
    target: "application/test_sample.py::test_doubles",
    test: "test_sample.py",
    function: "test_doubles",
};

const RUST: Layout = Layout {
    language: "rust",
    base: "release",
    prefix: "change/",
    source: "crates/engine",
    memory: "knowledge",
    skills: "policies",
    runner: "cargo",
    target: "tests::doubles",
    test: "src/lib.rs",
    function: "doubles",
};

fn consumer(program: impl AsRef<OsStr>, root: &Path, args: &[&str]) -> Run {
    let mut command = hermetic(program);
    command.args(args).current_dir(root);
    run(command.env("PATH", python_path()), "")
}

fn just(root: &Path, args: &[&str]) {
    let result = consumer("just", root, args);
    assert_eq!(result.code, Some(0), "just {args:?}: {}", result.output());
}

fn agentrig(root: &Path, args: &[&str], input: &str) -> Run {
    let launcher = root.join(".agentrig/bin/agentrig");
    run(
        launch(&launcher, root, args).env("PATH", python_path()),
        input,
    )
}

fn copy(from: &Path, to: &Path) {
    fs::create_dir_all(to).unwrap();
    for entry in fs::read_dir(from).unwrap() {
        let entry = entry.unwrap();
        let name = entry.file_name();
        let generated = GENERATED.iter().any(|ignored| name == *ignored);
        let directory = entry.path().is_dir();
        if generated {
            continue;
        } else if directory {
            copy(&entry.path(), &to.join(&name));
        } else {
            fs::copy(entry.path(), to.join(&name)).unwrap();
        }
    }
}

fn bind(root: &Path, layout: &Layout) {
    let oracle = format!(
        "I001: {{check: tests, runner: {}, target: '{}'}}\n",
        layout.runner, layout.target
    );
    configure(root, "oracles", &oracle);
    let memory = root.join(layout.memory);
    fs::create_dir(memory.join("Invariants")).unwrap();
    let index = memory.join("Invariants.md");
    let row = format!(
        "| [I001](Invariants/001.md) | Doubles correctly | [{}](../{}/{}) |\n",
        layout.function, layout.source, layout.test
    );
    fs::write(&index, fs::read_to_string(&index).unwrap() + &row).unwrap();
    fs::write(memory.join("Invariants/001.md"), DETAIL).unwrap();
}

fn install(layout: &Layout) -> (TempDir, PathBuf) {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().canonicalize().unwrap();
    git(&root, &["init", "-q", "-b", layout.base]);
    git(&root, &["config", "user.name", "Test"]);
    git(&root, &["config", "user.email", "test@example.invalid"]);
    fs::write(root.join(".gitignore"), IGNORED).unwrap();
    git(&root, &["add", "."]);
    git(&root, &["commit", "-qm", "base"]);
    let bootstrap = format!("{}bootstrap", layout.prefix);
    git(&root, &["switch", "-c", &bootstrap]);
    let (language, source, memory) = (layout.language, layout.source, layout.memory);
    let (skills, base, prefix) = (layout.skills, layout.base, layout.prefix);
    let initialized = invoke(
        &root,
        &[
            "init",
            "--language",
            language,
            "--source",
            source,
            "--memory",
            memory,
            "--skills",
            skills,
            "--base",
            base,
            "--prefix",
            prefix,
        ],
    );
    assert_eq!(initialized.code, Some(0), "{}", initialized.output());
    let copied = fs::read(installed_runtime(&root)).unwrap();
    assert!(copied == fs::read(worker()).unwrap());
    let examples = repository().join("Project/shared/testing/examples");
    copy(&examples.join(language).join(source), &root.join(source));
    bind(&root, layout);
    (temp, root)
}

fn ready(root: &Path) {
    let commands: [&[&str]; 7] = [
        &["config-check"],
        &["doctor"],
        &["resume"],
        &["memory-check"],
        &["check"],
        &["lint"],
        &["run", "test"],
    ];
    for command in commands {
        let result = agentrig(root, command, "");
        assert_eq!(result.code, Some(0), "{command:?}: {}", result.output());
    }
    just(root, &["config-check"]);
    just(root, &["run", "test"]);
}

fn structure_is_scored(root: &Path, layout: &Layout) {
    let skill = root.join(layout.skills).join("clarify-structure/SKILL.md");
    assert!(skill.is_file(), "{}", skill.display());
    let recipe = consumer("just", root, &["lint-structure", layout.source]);
    assert_eq!(recipe.code, Some(0), "{}", recipe.output());
    assert!(recipe.stdout.starts_with("total "), "{}", recipe.stdout);
    let direct = agentrig(root, &["lint-structure", layout.source], "");
    assert_eq!(recipe.stdout, direct.stdout);
}

fn unsupported_extension_is_rejected(root: &Path, layout: &Layout) {
    let config = root.join(".agentrig/lint.yaml");
    let before = fs::read_to_string(&config).unwrap();
    let script = root.join(layout.source).join("tool.sh");
    fs::write(script, "if true; then echo example; fi\n").unwrap();
    let skill = format!("{}/refactor-long-function/SKILL.md", layout.skills);
    let rule = format!(
        "\n- id: unsupported\n  kind: function-lines\n  target: file\n  include: [\"{}/**\"]\n  extensions: [\".sh\"]\n  warning: 40\n  warning_skill: \"{skill}\"\n  error_skill: \"{skill}\"\n",
        layout.source
    );
    fs::write(&config, before.clone() + &rule).unwrap();
    let rejected = agentrig(root, &["config-check"], "");
    assert_eq!(rejected.code, Some(2), "{}", rejected.output());
    let named = rejected.stderr.contains(".sh") && rejected.stderr.contains("handlers support");
    assert!(named, "{}", rejected.stderr);
    let supported = rule.replace("[\".sh\"]", "[\".rs\", \".py\"]");
    fs::write(&config, before + &supported).unwrap();
    let accepted = agentrig(root, &["config-check"], "");
    assert_eq!(accepted.code, Some(0), "{}", accepted.output());
}

fn hook(root: &Path, tool: &str, input: Value) -> String {
    let event = json!({"hook_event_name": "PreToolUse", "tool_name": tool,
        "tool_input": input, "cwd": root});
    let result = agentrig(root, &["hook"], &event.to_string());
    assert_eq!(result.code, Some(0), "{}", result.output());
    result.stdout
}

fn hooks(root: &Path, layout: &Layout) {
    let state = format!("{}/State.md", layout.memory);
    let guidance = hook(root, "Write", json!({"file_path": state}));
    let edit_state = format!("{}/edit-state/SKILL.md", layout.skills);
    assert!(guidance.contains(&edit_state), "{guidance}");
    let shell = |command: &str| hook(root, "exec_command", json!({"cmd": command}));
    assert!(shell("just run unknown").contains("deny"));
    assert!(shell("echo bypass").contains("deny"));
    assert!(!shell("just run test").contains("deny"));
    let entry = shell("just check --only lint");
    let reminded =
        entry.contains("deny") && entry.contains("edit-state/SKILL.md for VAC check entry");
    assert!(reminded, "{entry}");
    assert!(!shell("just check --only lint").contains("deny"));
    let skill = format!("{}/refactor-large-directory/SKILL.md", layout.skills);
    let edit = |name: &str| {
        let path = format!("{}/unit/{name}", layout.source);
        hook(root, "Write", json!({"file_path": path}))
    };
    let first = edit("first");
    assert!(first.contains("deny") && first.contains(&skill), "{first}");
    assert!(!edit("second").contains("deny"));
}

fn integrate(root: &Path, layout: &Layout) {
    git(root, &["add", "."]);
    just(root, &["check", "--staged"]);
    let committed = consumer("git", root, &["commit", "-qm", "attach scaffold"]);
    assert_eq!(committed.code, Some(0), "{}", committed.output());
    just(root, &["feature-merge"]);
    assert_eq!(git(root, &["branch", "--show-current"]), layout.base);
    let bootstrap = format!("{}bootstrap", layout.prefix);
    let deleted = git_run(root, &["branch", "-D", &bootstrap]);
    assert_ne!(deleted.code, Some(0), "{}", deleted.output());
    just(root, &["feature-start", "next"]);
    let next = format!("{}next", layout.prefix);
    assert_eq!(git(root, &["branch", "--show-current"]), next);
    assert!(!root.join("Project/Cargo.toml").exists());
    let repeated = agentrig(root, &["init"], "");
    assert_eq!(repeated.code, Some(2), "{}", repeated.output());
}

fn deliver(layout: &Layout) {
    let (_temp, root) = install(layout);
    ready(&root);
    structure_is_scored(&root, layout);
    unsupported_extension_is_rejected(&root, layout);
    hooks(&root, layout);
    integrate(&root, layout);
}

#[test]
fn python_project_is_delivered_in_an_independent_layout() {
    deliver(&PYTHON);
}

#[test]
fn rust_project_is_delivered_in_an_independent_layout() {
    deliver(&RUST);
}
