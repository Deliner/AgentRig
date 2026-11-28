use super::{consumer, flow, initialize, yaml};
use crate::shared::testing::{git, hermetic, invoke, launch, run, snapshot};
use std::{collections::BTreeSet, fs, path::Path};

const MODULES: [&str; 8] = [
    "project.yaml",
    "git.yaml",
    "commands.yaml",
    "checks.yaml",
    "hooks.yaml",
    "oracles.yaml",
    "tools.yaml",
    "rust.yaml",
];
const SERVICE: &str = "rig space's $cash";
const SESSION: &str = r#"{"hook_event_name": "SessionStart", "session_id": "custom-service"}"#;
const INVALID_LAYOUTS: [(Option<&str>, &[&str]); 5] = [
    (Some(".codex"), &[]),
    (Some(".claude"), &[]),
    (None, &["--skills", ".agentrig/bin/agentrig"]),
    (None, &["--skills", ".agents/skills"]),
    (None, &["--skills", ".claude"]),
];
const AGENT_SKILLS: [&str; 2] = [".agents/skills", ".claude/skills"];

fn refused(root: &Path, args: &[&str]) {
    let before = snapshot(root);
    assert!(initialize(root, args).is_err(), "{args:?}");
    assert_eq!(snapshot(root), before, "{args:?}");
}

fn adapters(root: &Path, service: &Path) {
    let installed = service.join("bin/agentrig");
    let doctor = run(&mut launch(&installed, root, &["doctor"]), "");
    assert_eq!(doctor.code, Some(0), "{}", doctor.output());
    let checked = run(hermetic("just").arg("config-check").current_dir(root), "");
    assert_eq!(checked.code, Some(0), "{}", checked.output());
    let hooks = fs::read_to_string(root.join(".codex/hooks.json")).unwrap();
    let hooks: serde_json::Value = serde_json::from_str(&hooks).unwrap();
    let command = hooks["hooks"]["SessionStart"][0]["hooks"][0]["command"].as_str();
    let mut session = hermetic("sh");
    session.args(["-c", command.unwrap()]).current_dir(root);
    let started = run(&mut session, SESSION);
    assert_eq!(started.code, Some(0), "{}", started.output());
    assert!(started.stdout.contains("complexity"), "{}", started.stdout);
    let blocked = run(
        hermetic(service.join("hooks/pre-commit")).current_dir(root),
        "",
    );
    assert_ne!(blocked.code, Some(0));
    assert!(blocked.stderr.contains("feature"), "{}", blocked.stderr);
    assert!(root.join(".cache/runtime/reminders").is_dir());
}

// DECISION: D040
// DECISION: D187
// INVARIANT: I022
#[test]
fn init_always_keeps_memory_in_ledger_and_sources_in_project() {
    let (_temp, root) = consumer();
    for option in ["--source", "--memory"] {
        refused(&root, &[option, "Other"]);
    }
    let result = invoke(&root, &["init"]);
    assert_eq!(result.code, Some(0), "{}", result.output());
    assert!(
        !result.stdout.contains("lint-baseline"),
        "{}",
        result.stdout
    );
    let paths = &yaml(&root.join(".agentrig/project.yaml"))["paths"];
    let layout = (&paths["memory"], &paths["sources"]);
    assert_eq!(layout, (&flow("Ledger"), &flow("[Project/**]")));
    assert!(root.join("Ledger/State.md").is_file());
}

// DECISION: D188
#[test]
fn init_links_each_agent_skill_directory_to_the_installed_skills() {
    let (_temp, root) = consumer();
    assert_eq!(initialize(&root, &[]).unwrap(), 0);
    let installed = fs::read_to_string(root.join(".agentrig/skills/repair/SKILL.md")).unwrap();
    for directory in AGENT_SKILLS {
        let target = fs::read_link(root.join(directory)).unwrap();
        assert_eq!(target, Path::new("../.agentrig/skills"), "{directory}");
        let seen = fs::read_to_string(root.join(directory).join("repair/SKILL.md")).unwrap();
        assert_eq!(seen, installed, "{directory}");
    }
}

// DECISION: D190
#[test]
fn init_starts_a_repository_without_commits_on_its_base_with_an_empty_commit() {
    for existing in [None, Some("trunk")] {
        let (_temp, root) = consumer();
        if let Some(branch) = existing {
            git(&root, &["init", "-q", "-b", branch]);
            fs::write(root.join("notes.txt"), "staged\n").unwrap();
            git(&root, &["add", "notes.txt"]);
        }
        assert_eq!(initialize(&root, &["--base", "main"]).unwrap(), 0);
        assert_eq!(git(&root, &["symbolic-ref", "HEAD"]), "refs/heads/main");
        let subjects = git(&root, &["log", "--format=%s", "main"]);
        assert_eq!(subjects, "Start the repository", "{existing:?}");
        let tree = git(&root, &["ls-tree", "-r", "--name-only", "main"]);
        assert_eq!(tree, "", "{existing:?}");
        let staged = git(&root, &["diff", "--cached", "--name-only"]);
        let expected = existing.map_or("", |_| "notes.txt");
        assert_eq!(staged, expected, "{existing:?}");
    }
}

// DECISION: D235
#[test]
fn init_names_lint_errors_in_existing_files_without_a_way_to_accept_them() {
    let (_temp, root) = consumer();
    fs::write(root.join("README.md"), format!("{}\n", "x".repeat(130))).unwrap();
    let result = invoke(&root, &["init"]);
    assert_eq!(result.code, Some(0), "{}", result.output());
    let named =
        "Lint errors in existing files: 1. The gate refuses a commit while lint reports an error;";
    assert!(result.stdout.contains(named), "{}", result.stdout);
    // DECISION: D212
    let recipes = fs::read_to_string(root.join(".agentrig/just/lint.just")).unwrap();
    assert!(!recipes.contains("lint-baseline"), "{recipes}");
    let help = invoke(&root, &["--help"]);
    assert!(!help.stdout.contains("lint-baseline"), "{}", help.stdout);
    let removed = invoke(&root, &["lint-baseline"]);
    let unknown = "unknown command \"lint-baseline\"";
    assert!(removed.stderr.contains(unknown), "{}", removed.output());
    assert_eq!(invoke(&root, &["lint"]).code, Some(1));
}

#[test]
fn init_rejects_invalid_layout_before_writing() {
    for (existing, args) in INVALID_LAYOUTS {
        let (_temp, root) = consumer();
        if let Some(name) = existing {
            fs::write(root.join(name), "user data").unwrap();
        }
        refused(&root, args);
    }
}

fn custom_service(language: &str) {
    let (_temp, root) = consumer();
    git(&root, &["init", "-q"]);
    let args = ["init", "--language", language, "--service", SERVICE];
    let result = invoke(&root, &args);
    assert_eq!(result.code, Some(0), "{}", result.output());
    let entries = fs::read_dir(root.join(".agentrig")).unwrap();
    let names = entries.map(|entry| entry.unwrap().file_name().into_string().unwrap());
    let expected = MODULES.into_iter().chain(["just"]).map(str::to_owned);
    assert_eq!(names.collect::<BTreeSet<_>>(), expected.collect());
    assert!(!root.join(".worker").exists());
    adapters(&root, &root.join(SERVICE));
}

// DECISION: D039
#[test]
fn custom_python_service_executes_installed_adapters() {
    custom_service("python");
}

// DECISION: D039
#[test]
fn custom_rust_service_executes_installed_adapters() {
    custom_service("rust");
}

#[test]
fn init_writes_the_queue_key_and_refuses_another_form_before_writing() {
    let (_temp, root) = consumer();
    refused(&root, &["--key", "rig-1"]);
    assert_eq!(initialize(&root, &["--key", "RIG"]).unwrap(), 0);
    let key = &yaml(&root.join(".agentrig/project.yaml"))["key"];
    assert_eq!(key, &flow("RIG"));
}

#[test]
fn invalid_service_leaves_consumer_untouched() {
    for service in ["../escape", "bad\npath", "{{injection}}"] {
        let (_temp, root) = consumer();
        refused(&root, &["--service", service]);
    }
}
