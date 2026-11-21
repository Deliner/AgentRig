use super::{expected, path, sources};
use crate::structural_lint::architecture::resolution::rust::Rust;
use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
};

#[test]
fn external_crates_do_not_hide_missing_anchored_paths() {
    let sources = sources(&[("lib.rs", "mod std { pub fn local() {} }")]);
    let external = BTreeSet::from(["std".into(), "dep".into()]);
    let resolver = Rust::new(Path::new("lib.rs"), &sources, &external, &BTreeMap::new()).unwrap();
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

// DECISION: D083
#[test]
fn clock_constructors_resolve_to_their_full_std_path() {
    let sources = sources(&[(
        "lib.rs",
        "use std::time::Instant; fn f() { let _ = Instant::now(); }",
    )]);
    let external = BTreeSet::from(["std".into()]);
    let resolver = Rust::new(Path::new("lib.rs"), &sources, &external, &BTreeMap::new()).unwrap();
    assert_eq!(
        resolver
            .resolve(Path::new("lib.rs"), &path("Instant::now"))
            .unwrap()
            .external,
        Some("std::time::Instant::now".into())
    );
    assert_eq!(
        resolver
            .resolve(Path::new("lib.rs"), &path("std::time::SystemTime::now"))
            .unwrap()
            .external,
        Some("std::time::SystemTime::now".into())
    );
}
