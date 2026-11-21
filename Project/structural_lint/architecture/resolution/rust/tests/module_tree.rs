use super::{expected, path, sources};
use crate::structural_lint::architecture::resolution::rust::Rust;
use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
};

#[test]
fn declared_module_tree_handles_both_filenames_and_inline_modules() {
    let sources = sources(&[
        ("src/lib.rs", "mod app; mod domain { pub mod api; }"),
        ("src/app.rs", "pub mod nested;"),
        ("src/app/nested/mod.rs", "pub fn run() {}"),
        ("src/domain/api.rs", "pub struct Item;"),
        ("src/unreferenced.rs", ""),
    ]);
    let external = BTreeSet::new();
    let resolver = Rust::new(
        Path::new("src/lib.rs"),
        &sources,
        &external,
        &BTreeMap::new(),
    )
    .unwrap();
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
fn missing_or_ambiguous_module_files_are_errors() {
    let external = BTreeSet::new();
    let missing = sources(&[("lib.rs", "mod absent;")]);
    assert!(Rust::new(Path::new("lib.rs"), &missing, &external, &BTreeMap::new()).is_err());
    let ambiguous = sources(&[
        ("lib.rs", "mod item;"),
        ("item.rs", ""),
        ("item/mod.rs", ""),
    ]);
    assert!(Rust::new(Path::new("lib.rs"), &ambiguous, &external, &BTreeMap::new()).is_err());
    let duplicate = sources(&[("lib.rs", "mod item {} mod item {}")]);
    assert!(Rust::new(Path::new("lib.rs"), &duplicate, &external, &BTreeMap::new()).is_err());
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
            Rust::new(Path::new("lib.rs"), &sources, &external, &BTreeMap::new()).is_err(),
            "{code}"
        );
    }
}

#[test]
fn invalid_or_ambiguous_module_paths_remain_incomplete() {
    for declaration in [
        "#[path = \"missing.rs\"] mod item;",
        "#[path = \"../outside.rs\"] mod item;",
        "#[path = \"/absolute.rs\"] mod item;",
        "#[path = \"lib.rs\"] mod recursive;",
        "#[path = \"item.rs\"] #[path = \"other.rs\"] mod item;",
        "#[path = concat!(\"item\", \".rs\")] mod item;",
        "#[path = \"item.rs\"] mod item {}",
        "mod inline { #[path = \"item.rs\"] mod item; }",
    ] {
        let sources = sources(&[("lib.rs", declaration), ("item.rs", "")]);
        assert!(
            Rust::new(
                Path::new("lib.rs"),
                &sources,
                &BTreeSet::new(),
                &BTreeMap::new()
            )
            .is_err(),
            "{declaration}"
        );
    }
}
