use super::{expected, path, sources};
use crate::structural_lint::architecture::extraction::Target;
use crate::structural_lint::architecture::resolution::rust::Rust;
use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
};

#[test]
fn an_external_reexport_still_depends_on_its_local_facade() {
    let sources = sources(&[
        ("lib.rs", "mod app; pub use dep::Item as Public;"),
        ("app.rs", "use crate::Public;"),
    ]);
    let external = BTreeSet::from(["dep".into()]);
    let resolver = Rust::new(Path::new("lib.rs"), &sources, &external, &BTreeMap::new()).unwrap();
    assert_eq!(
        resolver
            .resolve(Path::new("app.rs"), &path("Public::new"))
            .unwrap()
            .files,
        expected("lib.rs")
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
    let resolver = Rust::new(Path::new("lib.rs"), &sources, &external, &BTreeMap::new()).unwrap();
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
    let resolver = Rust::new(Path::new("lib.rs"), &sources, &external, &BTreeMap::new()).unwrap();
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
fn outside_its_parent_a_private_module_name_means_the_item_reexported_under_it() {
    let sources = sources(&[
        ("lib.rs", "mod options; mod app;"),
        ("options/mod.rs", "mod take; pub use take::take;"),
        ("options/take.rs", "pub fn take() {}"),
        ("app.rs", "use crate::options::take;"),
    ]);
    let external = BTreeSet::new();
    let resolver = Rust::new(Path::new("lib.rs"), &sources, &external, &BTreeMap::new()).unwrap();
    for (source, name, file) in [
        ("app.rs", "crate::options::take", "options/mod.rs"),
        ("options/mod.rs", "take::take", "options/take.rs"),
    ] {
        assert_eq!(
            resolver
                .resolve(Path::new(source), &path(name))
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
    let resolver = Rust::new(Path::new("lib.rs"), &sources, &external, &BTreeMap::new()).unwrap();
    assert!(
        resolver
            .resolve(Path::new("lib.rs"), &path("A::run"))
            .unwrap_err()
            .to_string()
            .contains("cyclic")
    );
}

#[test]
fn function_locals_are_not_mistaken_for_module_bindings() {
    let sources = sources(&[(
        "lib.rs",
        "fn a() { use std::io as name; struct Local; } fn b() {}",
    )]);
    assert!(sources[Path::new("lib.rs")].rust_imports.is_empty());
    let external = BTreeSet::from(["std".into()]);
    let resolver = Rust::new(Path::new("lib.rs"), &sources, &external, &BTreeMap::new()).unwrap();
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
