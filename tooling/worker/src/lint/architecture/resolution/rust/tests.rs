use super::*;
use crate::lint::architecture::source;
use std::{fs, process::Command};

fn sources(files: &[(&str, &str)]) -> BTreeMap<PathBuf, References> {
    files
        .iter()
        .map(|(path, text)| {
            (
                PathBuf::from(path),
                source::extract(Path::new(path), text).unwrap(),
            )
        })
        .collect()
}

fn path(value: &str) -> Target {
    Target::RustPath {
        path: value.into(),
        scope: Vec::new(),
    }
}

fn expected(file: &str) -> BTreeSet<PathBuf> {
    BTreeSet::from([file.into()])
}

#[test]
fn an_external_reexport_still_depends_on_its_local_facade() {
    let sources = sources(&[
        ("lib.rs", "mod app; pub use dep::Item as Public;"),
        ("app.rs", "use crate::Public;"),
    ]);
    let external = BTreeSet::from(["dep".into()]);
    let resolver = Rust::new(Path::new("lib.rs"), &sources, &external).unwrap();
    assert_eq!(
        resolver
            .resolve(Path::new("app.rs"), &path("Public::new"))
            .unwrap()
            .files,
        expected("lib.rs")
    );
}

#[test]
fn declared_module_tree_handles_both_filenames_and_inline_modules() {
    let sources = sources(&[
        ("src/lib.rs", "mod app; mod domain { pub mod api; }"),
        ("src/app.rs", "mod nested;"),
        ("src/app/nested/mod.rs", "pub fn run() {}"),
        ("src/domain/api.rs", "pub struct Item;"),
        ("src/unreferenced.rs", ""),
    ]);
    let external = BTreeSet::new();
    let resolver = Rust::new(Path::new("src/lib.rs"), &sources, &external).unwrap();
    for (name, file) in [
        ("crate::app::nested::run", "src/app/nested/mod.rs"),
        ("crate::domain::api::Item", "src/domain/api.rs"),
        ("crate::domain", "src/lib.rs"),
    ] {
        assert_eq!(
            resolver
                .resolve(Path::new("src/lib.rs"), &path(name))
                .unwrap()
                .files,
            expected(file)
        );
    }
    assert!(
        resolver
            .resolve(Path::new("src/unreferenced.rs"), &path("crate::app"))
            .is_err()
    );
    assert!(
        resolver
            .resolve(Path::new("src/lib.rs"), &path("crate::unreferenced"))
            .is_err()
    );
}

#[test]
fn self_super_and_repeated_super_use_the_actual_module_scope() {
    let sources = sources(&[
        ("lib.rs", "mod app { pub mod nested; } pub fn root() {}"),
        (
            "app/nested.rs",
            "pub fn local() {} mod inline { pub fn work() {} }",
        ),
    ]);
    let external = BTreeSet::new();
    let resolver = Rust::new(Path::new("lib.rs"), &sources, &external).unwrap();
    assert_eq!(
        resolver
            .resolve(Path::new("app/nested.rs"), &path("self::local"))
            .unwrap()
            .files,
        expected("app/nested.rs")
    );
    let nested = Target::RustPath {
        path: "super::super::super::root".into(),
        scope: vec!["inline".into()],
    };
    assert_eq!(
        resolver
            .resolve(Path::new("app/nested.rs"), &nested)
            .unwrap()
            .files,
        expected("lib.rs")
    );
    assert!(
        resolver
            .resolve(Path::new("lib.rs"), &path("super::root"))
            .is_err()
    );
}

#[test]
fn grouped_aliases_reach_submodules_and_item_reexports_preserve_facades() {
    let sources = sources(&[
        (
            "lib.rs",
            "mod domain; mod app; pub use crate::domain::Item as Public;",
        ),
        ("domain.rs", "pub struct Item; pub mod private;"),
        ("domain/private.rs", "pub fn work() {}"),
        (
            "app.rs",
            "use crate::domain::{self as api, Item as Alias}; pub fn run() { api::private::work(); }",
        ),
    ]);
    let external = BTreeSet::new();
    let resolver = Rust::new(Path::new("lib.rs"), &sources, &external).unwrap();
    for (name, file) in [
        ("api::private::work", "domain/private.rs"),
        ("Alias::new", "domain.rs"),
        ("crate::Public", "lib.rs"),
    ] {
        assert_eq!(
            resolver
                .resolve(Path::new("app.rs"), &path(name))
                .unwrap()
                .files,
            expected(file)
        );
    }
}

#[test]
fn aliases_cannot_silently_form_a_resolution_cycle() {
    let sources = sources(&[("lib.rs", "use self::B as A; use self::A as B;")]);
    let external = BTreeSet::new();
    let resolver = Rust::new(Path::new("lib.rs"), &sources, &external).unwrap();
    assert!(
        resolver
            .resolve(Path::new("lib.rs"), &path("A::run"))
            .unwrap_err()
            .to_string()
            .contains("cyclic")
    );
}

#[test]
fn missing_or_ambiguous_module_files_are_errors() {
    let external = BTreeSet::new();
    let missing = sources(&[("lib.rs", "mod absent;")]);
    assert!(Rust::new(Path::new("lib.rs"), &missing, &external).is_err());
    let ambiguous = sources(&[
        ("lib.rs", "mod item;"),
        ("item.rs", ""),
        ("item/mod.rs", ""),
    ]);
    assert!(Rust::new(Path::new("lib.rs"), &ambiguous, &external).is_err());
    let duplicate = sources(&[("lib.rs", "mod item {} mod item {}")]);
    assert!(Rust::new(Path::new("lib.rs"), &duplicate, &external).is_err());
}

#[test]
fn external_crates_do_not_hide_missing_anchored_paths() {
    let sources = sources(&[("lib.rs", "mod std { pub fn local() {} }")]);
    let external = BTreeSet::from(["std".into(), "dep".into()]);
    let resolver = Rust::new(Path::new("lib.rs"), &sources, &external).unwrap();
    assert_eq!(
        resolver
            .resolve(Path::new("lib.rs"), &path("std::local"))
            .unwrap()
            .files,
        expected("lib.rs")
    );
    assert!(
        resolver
            .resolve(Path::new("lib.rs"), &path("::std::fmt"))
            .unwrap()
            .external
            .is_some()
    );
    assert!(
        resolver
            .resolve(Path::new("lib.rs"), &path("dep::Item"))
            .unwrap()
            .external
            .is_some()
    );
    assert!(
        resolver
            .resolve(Path::new("lib.rs"), &path("crate::dep::Item"))
            .is_err()
    );
    assert!(
        resolver
            .resolve(Path::new("lib.rs"), &path("unknown::Item"))
            .is_err()
    );
}

#[test]
fn unsupported_source_cannot_build_a_verified_module_tree() {
    let external = BTreeSet::new();
    for code in [
        "#[path = \"other.rs\"] mod a;",
        "fn f() { mod a {} }",
        "fn f() { call!(crate::hidden::run()); }",
        "use crate::{;",
    ] {
        let sources = sources(&[("lib.rs", code)]);
        assert!(
            Rust::new(Path::new("lib.rs"), &sources, &external).is_err(),
            "{code}"
        );
    }
}

#[test]
fn function_locals_are_not_mistaken_for_module_bindings() {
    let sources = sources(&[(
        "lib.rs",
        "fn a() { use std::io as name; struct Local; } fn b() {}",
    )]);
    assert!(sources[Path::new("lib.rs")].rust_imports.is_empty());
    let external = BTreeSet::from(["std".into()]);
    let resolver = Rust::new(Path::new("lib.rs"), &sources, &external).unwrap();
    assert!(
        resolver
            .resolve(Path::new("lib.rs"), &path("name::Read"))
            .is_err()
    );
    assert!(
        resolver
            .resolve(Path::new("lib.rs"), &path("Local::new"))
            .is_err()
    );
}

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
    let resolver = Rust::new(Path::new("main.rs"), &sources, &external).unwrap();
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
        fs::write(project.path().join(file), text).unwrap();
    }
    let output = Command::new("rustc")
        .args(["+1.98.1", "--edition=2021", "main.rs", "-o", "consumer"])
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
