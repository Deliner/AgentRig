use super::{consumer, flow, initialize, yaml};
use crate::shared::testing::{invoke, write};
use serde_yaml_ng::Value;
use std::{fs, path::Path};

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

fn rust(root: &Path, args: &[&str]) {
    let args = [&["--language", "rust"][..], args].concat();
    assert_eq!(initialize(root, &args).unwrap(), 0);
}

fn touch(root: &Path, paths: &[&str]) {
    for path in paths {
        write(root, path, "");
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

#[test]
fn rust_init_derives_a_binary_crate_from_the_root_manifest() {
    let (_temp, root) = consumer();
    package(&root, "tool", "main.rs");
    rust(&root, &[]);
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
        rust(&root, &[]);
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
    rust(&root, &[]);
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
    rust(&root, &[]);
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
    rust(&root, &[]);
    let crates = "{x: crates/shared/lib.rs}";
    assert_layout(&root, "[crates/shared/lib.rs]", crates, ROOT_TESTS);
}

#[test]
fn rust_init_skips_member_globs_over_missing_directories() {
    let (_temp, root) = consumer();
    let workspace = "[workspace]\nmembers = [\"crates/*\", \"missing/*\"]\n";
    fs::write(root.join("Cargo.toml"), workspace).unwrap();
    package(&root.join("crates/a"), "a", "lib.rs");
    rust(&root, &[]);
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
    rust(&root, &[]);
    let crates = "{a: crates/a/src/lib.rs}";
    assert_layout(&root, "[crates/a/src/lib.rs]", crates, ROOT_TESTS);
}
