use super::{expected, path, sources};
use crate::structural_lint::architecture::extraction::{self, References, Target};
use crate::structural_lint::architecture::resolution::rust::Rust;
use std::{
    collections::{BTreeMap, BTreeSet},
    path::{Path, PathBuf},
};

// DECISION: D136
fn expansion_tree() -> BTreeMap<PathBuf, References> {
    sources(&[
        (
            "lib.rs",
            "mod app; mod wire { include!(\"parts/wire.rs\"); } mod out { include!(concat!(env!(\"OUT_DIR\"), \"/gen.rs\")); } mod a { pub use super::b::*; } mod b { pub use super::a::*; } macro_rules! make { () => {} } pub fn root() {}",
        ),
        ("parts/wire.rs", "pub fn connect() {}"),
        ("app.rs", "use super::*; use dep::*;"),
    ])
}

// DECISION: D136
#[test]
fn globs_and_generated_names_resolve_to_their_owner() {
    let tree = expansion_tree();
    let external = BTreeSet::from(["std".into(), "dep".into()]);
    let resolver = Rust::new(Path::new("lib.rs"), &tree, &external, &BTreeMap::new()).unwrap();
    let resolve = |file: &str, name: &str| resolver.resolve(Path::new(file), &path(name));
    for (file, name, owner) in [
        ("app.rs", "root", "lib.rs"),
        ("parts/wire.rs", "crate::root", "lib.rs"),
        ("lib.rs", "crate::wire::connect", "lib.rs"),
        ("lib.rs", "crate::out::Generated", "lib.rs"),
    ] {
        assert_eq!(
            resolve(file, name).unwrap().files,
            expected(owner),
            "{name}"
        );
    }
    for (name, origin) in [
        ("Item::new", "dep::Item::new"),
        ("Option::Some", "::std::option::Option::Some"),
    ] {
        assert_eq!(
            resolve("app.rs", name).unwrap().external.as_deref(),
            Some(origin)
        );
    }
    assert!(resolve("lib.rs", "crate::a::missing").is_err());
}

// DECISION: D136
#[test]
fn includes_and_local_macros_resolve_to_their_files() {
    let tree = expansion_tree();
    let external = BTreeSet::from(["std".into(), "dep".into()]);
    let resolver = Rust::new(Path::new("lib.rs"), &tree, &external, &BTreeMap::new()).unwrap();
    let include = Target::RustInclude {
        file: "parts/wire.rs".into(),
        scope: vec!["wire".into()],
    };
    let included = resolver.resolve(Path::new("lib.rs"), &include).unwrap();
    assert_eq!(included.files, expected("parts/wire.rs"));
    let local = Target::RustMacro(extraction::RustMacro {
        path: "make".into(),
        scope: Vec::new(),
        literal: None,
    });
    let defined = resolver.resolve(Path::new("app.rs"), &local).unwrap();
    assert_eq!(defined.files, expected("lib.rs"));
    let missing = sources(&[("lib.rs", "include!(\"absent.rs\");")]);
    let absent = Target::RustInclude {
        file: "absent.rs".into(),
        scope: Vec::new(),
    };
    let resolver = Rust::new(Path::new("lib.rs"), &missing, &external, &BTreeMap::new()).unwrap();
    assert!(resolver.resolve(Path::new("lib.rs"), &absent).is_err());
}
