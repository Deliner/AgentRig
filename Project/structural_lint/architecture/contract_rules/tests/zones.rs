use super::{messages, write};
use crate::structural_lint::architecture::contract_rules::{Contracts, EnvironmentUse, effects};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::PathBuf,
};

// DECISION: D078
#[test]
fn environment_namespaces_match_only_at_a_path_boundary() {
    for external in [
        "std::fs",
        "std::fs::read",
        "::std::env::var",
        "std::process::Command",
    ] {
        assert!(effects::environment(external), "{external}");
    }
    for external in [
        "std::collections::BTreeMap",
        "std::path::PathBuf",
        "std::fmt",
        "std::env_logger",
        "fs::read",
        "crate::fs::read",
    ] {
        assert!(!effects::environment(external), "{external}");
    }
}

// DECISION: D078
#[test]
fn zone_pure_directories_reject_environment_access_in_their_own_files() {
    let temp = tempfile::tempdir().unwrap();
    write(
        temp.path(),
        "Project/calc",
        "purpose: Pure numeric evaluation\nzone: pure",
    );
    write(temp.path(), "Project/io", "purpose: Filesystem boundary");
    let directories: BTreeSet<_> = ["Project/calc", "Project/io"]
        .into_iter()
        .map(PathBuf::from)
        .collect();
    let loaded = Contracts::load(temp.path(), &directories);
    assert!(loaded.issues.is_empty(), "{:?}", messages(&loaded.issues));
    let use_site = |line, path: &str| EnvironmentUse {
        line,
        path: path.into(),
    };
    let mut environment = BTreeMap::new();
    environment.insert(
        PathBuf::from("Project/calc/math.rs"),
        vec![use_site(3, "std::fs::read"), use_site(9, "std::env::var")],
    );
    environment.insert(
        PathBuf::from("Project/io/sink.rs"),
        vec![use_site(1, "std::fs")],
    );
    let found = loaded.contracts.check_zones(&environment);
    let located: Vec<_> = found
        .iter()
        .map(|issue| (issue.path.display().to_string(), issue.line))
        .collect();
    let calc = "Project/calc/math.rs".to_owned();
    assert_eq!(located, [(calc.clone(), Some(3)), (calc, Some(9))]);
    assert!(
        found
            .iter()
            .all(|issue| issue.message.contains("must not access the environment"))
    );
}

// DECISION: D083
#[test]
fn clock_reads_match_only_the_now_constructors() {
    for external in [
        "std::time::Instant::now",
        "std::time::SystemTime::now",
        "::std::time::SystemTime::now",
    ] {
        assert!(effects::clock(external), "{external}");
    }
    for external in [
        "std::time::Duration",
        "std::time::Instant",
        "std::time::SystemTime::UNIX_EPOCH",
        "std::time::Duration::from_secs",
    ] {
        assert!(!effects::clock(external), "{external}");
    }
}

// DECISION: D083
#[test]
fn zone_pure_directories_reject_wall_clock_reads_in_their_own_files() {
    let temp = tempfile::tempdir().unwrap();
    write(
        temp.path(),
        "Project/calc",
        "purpose: Pure numeric evaluation\nzone: pure",
    );
    let directories = BTreeSet::from([PathBuf::from("Project/calc")]);
    let loaded = Contracts::load(temp.path(), &directories);
    assert!(loaded.issues.is_empty(), "{:?}", messages(&loaded.issues));
    let use_site = |line, path: &str| EnvironmentUse {
        line,
        path: path.into(),
    };
    let mut environment = BTreeMap::new();
    environment.insert(
        PathBuf::from("Project/calc/clock.rs"),
        vec![
            use_site(4, "std::time::Instant::now"),
            use_site(8, "std::time::SystemTime::now"),
        ],
    );
    let found = loaded.contracts.check_zones(&environment);
    let lines: Vec<_> = found.iter().map(|issue| issue.line).collect();
    assert_eq!(lines, [Some(4), Some(8)]);
    assert!(
        found
            .iter()
            .all(|issue| issue.message.contains("must not read the wall clock"))
    );
}

// DECISION: D104
#[test]
fn pure_zone_effects_name_entropy_network_threads_and_console() {
    let cases = [
        ("std::fs::read", Some("access the environment")),
        ("std::time::Instant::now", Some("read the wall clock")),
        ("rand::random", Some("draw entropy")),
        ("rand::thread_rng", Some("draw entropy")),
        ("::rand::rng", Some("draw entropy")),
        ("getrandom::getrandom", Some("draw entropy")),
        ("getrandom::fill", Some("draw entropy")),
        ("std::net", Some("use the network")),
        ("std::net::TcpStream::connect", Some("use the network")),
        ("std::thread::spawn", Some("use threads")),
        ("std::io::stdin", Some("use the console")),
        ("std::io::stdout", Some("use the console")),
        ("std::io::stderr", Some("use the console")),
        ("std::print", Some("use the console")),
        ("std::println", Some("use the console")),
        ("std::eprint", Some("use the console")),
        ("std::eprintln", Some("use the console")),
        ("rand::rngs::StdRng", None),
        ("rand::Rng", None),
        ("getrandom::Error", None),
        ("std::io::Write", None),
        ("std::io::Error", None),
        ("std::sync::Mutex", None),
        ("std::format", None),
        ("std::dbg", None),
    ];
    for (external, expected) in cases {
        assert_eq!(effects::effect(external), expected, "{external}");
    }
}

// DECISION: D104
#[test]
fn zone_pure_directories_name_the_effect_of_each_forbidden_reference() {
    let temp = tempfile::tempdir().unwrap();
    write(
        temp.path(),
        "Project/calc",
        "purpose: Pure numeric evaluation\nzone: pure",
    );
    let directories = BTreeSet::from([PathBuf::from("Project/calc")]);
    let loaded = Contracts::load(temp.path(), &directories);
    assert!(loaded.issues.is_empty(), "{:?}", messages(&loaded.issues));
    let use_site = |line, path: &str| EnvironmentUse {
        line,
        path: path.into(),
    };
    let mut environment = BTreeMap::new();
    environment.insert(
        PathBuf::from("Project/calc/effects.rs"),
        vec![
            use_site(2, "rand::random"),
            use_site(5, "std::net::TcpStream::connect"),
            use_site(7, "std::thread::spawn"),
            use_site(9, "std::println"),
        ],
    );
    let found = loaded.contracts.check_zones(&environment);
    let expected = [
        "draw entropy",
        "use the network",
        "use threads",
        "use the console",
    ];
    assert_eq!(found.len(), expected.len(), "{:?}", messages(&found));
    for (issue, effect) in found.iter().zip(expected) {
        let named = issue.message.contains(&format!("must not {effect};"));
        assert!(named, "{}", issue.message);
    }
}
