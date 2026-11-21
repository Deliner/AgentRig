use super::{expected, path, sources};
use crate::shared::testing::write;
use crate::structural_lint::architecture::resolution::rust::Rust;
use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
    process::Command,
};

#[test]
fn compiled_consumer_uses_the_resolved_files() {
    let files = [
        (
            "main.rs",
            "mod app; mod domain; fn main() { let valid = app::run() == 42; if !valid { std::process::exit(1); } }",
        ),
        (
            "app.rs",
            "use crate::domain as api; pub fn run() -> i32 { api::value() }",
        ),
        ("domain.rs", "pub fn value() -> i32 { 42 }"),
    ];
    let sources = sources(&files);
    let external = BTreeSet::from(["std".into()]);
    let resolver = Rust::new(Path::new("main.rs"), &sources, &external, &BTreeMap::new()).unwrap();
    assert_eq!(
        resolver
            .resolve(Path::new("app.rs"), &path("api::value"))
            .unwrap()
            .files,
        expected("domain.rs")
    );
    for (file, references) in &sources {
        for reference in &references.items {
            resolver.resolve(file, &reference.target).unwrap();
        }
    }
    compile_and_run(&files);
}

fn compile_and_run(files: &[(&str, &str)]) {
    let project = tempfile::tempdir().unwrap();
    for (file, text) in files {
        write(project.path(), file, text);
    }
    let output = Command::new("rustc")
        .args(["--edition=2021", "main.rs", "-o", "consumer"])
        .current_dir(project.path())
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        Command::new(project.path().join("consumer"))
            .status()
            .unwrap()
            .success()
    );
}

#[test]
fn literal_module_paths_match_compiler_file_and_child_resolution() {
    let files = [
        (
            "main.rs",
            "#[path = r#\"fixtures/shared.rs\"#] mod shared; mod app; fn main() { assert_eq!(app::value(), 42); }",
        ),
        ("fixtures/shared.rs", "pub mod nested;"),
        ("fixtures/nested.rs", "pub fn value() -> i32 { 42 }"),
        (
            "app.rs",
            "#[allow(dead_code)] #[path = \"fixtures/../other.rs\"] pub mod other; pub fn value() -> i32 { other::value() }",
        ),
        (
            "other.rs",
            "pub fn value() -> i32 { crate::shared::nested::value() }",
        ),
    ];
    let sources = sources(&files);
    let resolver = Rust::new(
        Path::new("main.rs"),
        &sources,
        &BTreeSet::new(),
        &BTreeMap::new(),
    )
    .unwrap();
    for (name, file) in [
        ("shared::nested::value", "fixtures/nested.rs"),
        ("app::other::value", "other.rs"),
    ] {
        assert_eq!(
            resolver
                .resolve(Path::new("main.rs"), &path(name))
                .unwrap()
                .files,
            expected(file)
        );
    }
    compile_and_run(&files);
}
