use crate::project_setup::initialization::init;
use crate::shared::testing::{git, hermetic, invoke, launch, run, snapshot};
use anyhow::Result;
use serde_yaml_ng::Value;
use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
};
use tempfile::TempDir;

const AGENTRIG: &str = r#"[package]
name = "agentrig"

[lib]
path = "lib.rs"

[[bin]]
name = "agentrig"
path = "cli_dispatch/main.rs"

[[bin]]
name = "agentrig-lint"
path = "structural_lint/standalone.rs"
"#;
const PARENT_PATHS: &str = r#"[package]
name = "x"

[lib]
path = "../shared/lib.rs"

[[bin]]
name = "outside"
path = "../../../outside.rs"
"#;
const TARGETS: &str = r#"[[example]]
name = "demo"
path = "examples/demo.rs"
[[test]]
name = "flow"
path = "tests/flow.rs"
[dependencies]
serde-json = "1"
[dev-dependencies]
proptest = "1"
[build-dependencies]
prost-build = "1"
[target.'cfg(unix)'.dependencies]
libc = "0.2"
"#;
const ROOT_TESTS: &str = "[cargo, test, --manifest-path, Cargo.toml]";
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
const INVALID_LAYOUTS: [(Option<&str>, &[&str]); 3] = [
    (None, &["--source", "src/["]),
    (Some(".codex"), &[]),
    (None, &["--skills", ".agentrig/bin/agentrig"]),
];

fn consumer() -> (TempDir, PathBuf) {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().canonicalize().unwrap();
    (temp, root)
}

fn initialize(root: &Path, args: &[&str]) -> Result<i32> {
    let args: Vec<String> = args.iter().copied().map(str::to_owned).collect();
    init(root, &args)
}

fn rust(root: &Path, args: &[&str]) {
    let args = [&["--language", "rust"][..], args].concat();
    assert_eq!(initialize(root, &args).unwrap(), 0);
}

fn yaml(path: &Path) -> Value {
    serde_yaml_ng::from_str(&fs::read_to_string(path).unwrap()).unwrap()
}

fn flow(text: &str) -> Value {
    serde_yaml_ng::from_str(text).unwrap()
}

fn touch(root: &Path, paths: &[&str]) {
    for path in paths {
        let path = root.join(path);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, "").unwrap();
    }
}

fn package(directory: &Path, name: &str, root: &str) {
    touch(directory, &[&format!("src/{root}")]);
    let manifest = format!("[package]\nname = \"{name}\"\nversion = \"0.1.0\"\n");
    fs::write(directory.join("Cargo.toml"), manifest).unwrap();
}

fn architecture(root: &Path) -> Value {
    let lint = yaml(&root.join(".agentrig/lint.yaml"));
    let rules = lint["rules"].as_sequence().unwrap();
    let rule = rules.iter().find(|rule| rule["id"] == "architecture");
    rule.unwrap()["architecture"].clone()
}

fn assert_layout(root: &Path, roots: &str, crates: &str, tests: &str) {
    let settings = architecture(root);
    let derived = settings.get("rust_crates").cloned();
    let derived = derived.unwrap_or_else(|| flow("{}"));
    let commands = yaml(&root.join(".agentrig/commands.yaml"));
    let found = (&settings["rust_roots"], &derived, &commands["test"]["argv"]);
    assert_eq!(found, (&flow(roots), &flow(crates), &flow(tests)));
}

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
    assert!(service.join("runtime/reminders").is_dir());
}

// DECISION: D040
#[test]
fn default_layout_keeps_memory_in_ledger_and_sources_in_project() {
    let (_temp, root) = consumer();
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

// DECISION: D123
#[test]
fn init_offers_a_lint_baseline_for_errors_in_existing_files() {
    let (_temp, root) = consumer();
    fs::write(root.join("README.md"), format!("{}\n", "x".repeat(130))).unwrap();
    let result = invoke(&root, &["init"]);
    assert_eq!(result.code, Some(0), "{}", result.output());
    let offered = ["Lint errors in existing files: 1.", "just lint-baseline"];
    for text in offered {
        assert!(result.stdout.contains(text), "{}", result.stdout);
    }
    let recipes = fs::read_to_string(root.join("just/lint.just")).unwrap();
    assert!(recipes.contains("lint-baseline *args:"), "{recipes}");
    let codes = ["lint", "lint-baseline", "lint"].map(|command| invoke(&root, &[command]).code);
    assert_eq!(codes, [Some(1), Some(0), Some(0)]);
}

#[test]
fn rust_init_derives_a_binary_crate_from_the_root_manifest() {
    let (_temp, root) = consumer();
    package(&root, "tool", "main.rs");
    rust(&root, &["--source", "src"]);
    assert_layout(&root, "[src/main.rs]", "{}", ROOT_TESTS);
    let lint = invoke(&root, &["lint", "--json"]);
    let findings: serde_json::Value = serde_json::from_str(&lint.stdout).unwrap();
    let unresolved = [
        "crate root is missing",
        "no successfully analyzed Rust crate root",
    ];
    for finding in findings.as_array().unwrap() {
        let message = finding["message"].as_str().unwrap();
        let reported = unresolved.iter().any(|text| message.contains(text));
        assert!(!reported, "{message}");
    }
}

#[test]
fn rust_init_derives_workspace_member_roots() {
    for (name, library) in [("a", "a"), ("core-a", "core_a")] {
        let (_temp, root) = consumer();
        let workspace = "[workspace]\nmembers = [\"crates/*\"]\n";
        fs::write(root.join("Cargo.toml"), workspace).unwrap();
        package(&root.join("crates/a"), name, "lib.rs");
        package(&root.join("crates/b"), "b", "main.rs");
        rust(&root, &["--source", "crates"]);
        let roots = "[crates/a/src/lib.rs, crates/b/src/main.rs]";
        let crates = format!("{{{library}: crates/a/src/lib.rs}}");
        assert_layout(&root, roots, &crates, ROOT_TESTS);
    }
}

// DECISION: D136
#[test]
fn rust_init_adds_target_roots_build_script_and_dependency_externals() {
    let (_temp, root) = consumer();
    package(&root, "tool", "lib.rs");
    let manifest = root.join("Cargo.toml");
    let text = fs::read_to_string(&manifest).unwrap() + TARGETS;
    fs::write(&manifest, text).unwrap();
    touch(&root, &["examples/demo.rs", "tests/flow.rs", "build.rs"]);
    rust(&root, &["--source", "src"]);
    let settings = architecture(&root);
    let roots = flow("[src/lib.rs, examples/demo.rs, tests/flow.rs, build.rs]");
    assert_eq!(settings["rust_roots"], roots);
    let external = "{rust: [alloc, core, libc, proptest, prost_build, serde_json, std]}";
    assert_eq!(settings["external"], flow(external));
}

#[test]
fn rust_init_keeps_the_agentrig_layout() {
    let (_temp, root) = consumer();
    let roots = [
        "Project/lib.rs",
        "Project/cli_dispatch/main.rs",
        "Project/structural_lint/standalone.rs",
    ];
    touch(&root, &roots);
    fs::write(root.join("Project/Cargo.toml"), AGENTRIG).unwrap();
    rust(&root, &[]);
    let roots = format!("[{}]", roots.join(", "));
    let tests = "[cargo, test, --manifest-path, Project/Cargo.toml]";
    assert_layout(&root, &roots, "{agentrig: Project/lib.rs}", tests);
}

#[test]
fn rust_init_expands_recursive_member_globs() {
    let (_temp, root) = consumer();
    let workspace = "[workspace]\nmembers = [\"crates/**\"]\n";
    fs::write(root.join("Cargo.toml"), workspace).unwrap();
    package(&root.join("crates/a/b"), "b", "lib.rs");
    package(&root.join("crates/c"), "c", "main.rs");
    rust(&root, &["--source", "crates"]);
    let roots = "[crates/a/b/src/lib.rs, crates/c/src/main.rs]";
    assert_layout(&root, roots, "{b: crates/a/b/src/lib.rs}", ROOT_TESTS);
}

#[test]
fn rust_init_accepts_parent_paths_inside_the_root() {
    let (_temp, outer) = consumer();
    let root = outer.join("repo");
    touch(&root, &["crates/shared/lib.rs"]);
    touch(&outer, &["outside.rs"]);
    fs::create_dir(root.join("crates/x")).unwrap();
    let workspace = "[workspace]\nmembers = [\"crates/x\"]\n";
    fs::write(root.join("Cargo.toml"), workspace).unwrap();
    fs::write(root.join("crates/x/Cargo.toml"), PARENT_PATHS).unwrap();
    rust(&root, &["--source", "crates"]);
    let crates = "{x: crates/shared/lib.rs}";
    assert_layout(&root, "[crates/shared/lib.rs]", crates, ROOT_TESTS);
}

#[test]
fn rust_init_skips_member_globs_over_missing_directories() {
    let (_temp, root) = consumer();
    let workspace = "[workspace]\nmembers = [\"crates/*\", \"missing/*\"]\n";
    fs::write(root.join("Cargo.toml"), workspace).unwrap();
    package(&root.join("crates/a"), "a", "lib.rs");
    rust(&root, &["--source", "crates"]);
    let crates = "{a: crates/a/src/lib.rs}";
    assert_layout(&root, "[crates/a/src/lib.rs]", crates, ROOT_TESTS);
}

#[test]
fn rust_init_skips_excluded_workspace_members() {
    let (_temp, root) = consumer();
    let workspace = "[workspace]\nmembers = [\"crates/*\"]\nexclude = [\"crates/skip\"]\n";
    fs::write(root.join("Cargo.toml"), workspace).unwrap();
    package(&root.join("crates/a"), "a", "lib.rs");
    package(&root.join("crates/skip"), "skip", "lib.rs");
    rust(&root, &["--source", "crates"]);
    let crates = "{a: crates/a/src/lib.rs}";
    assert_layout(&root, "[crates/a/src/lib.rs]", crates, ROOT_TESTS);
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
    let expected = MODULES.map(str::to_owned);
    assert_eq!(names.collect::<BTreeSet<_>>(), BTreeSet::from(expected));
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
fn invalid_service_leaves_consumer_untouched() {
    for service in ["../escape", "bad\npath", "{{injection}}"] {
        let (_temp, root) = consumer();
        refused(&root, &["--service", service]);
    }
}
