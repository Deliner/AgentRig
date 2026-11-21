use super::{edge, messages};
use crate::structural_lint::architecture::contract_rules::graph;
use std::path::PathBuf;

#[test]
fn cycles_report_the_closing_measured_edge() {
    assert!(graph::cycles(&[]).is_empty());
    let mut edges = vec![edge("a/api.rs", "b/api.py"), edge("b/api.py", "c/api.ts")];
    assert!(graph::cycles(&edges).is_empty());
    edges.push(edge("c/api.ts", "a/api.rs"));
    let issues = graph::cycles(&edges);
    assert_eq!(
        messages(&issues),
        [
            "directory dependency cycle: a -> b -> c -> a; a/api.rs:7 uses b/api.py; b/api.py:7 uses c/api.ts; c/api.ts:7 uses a/api.rs"
        ]
    );
    assert_eq!(issues[0].path, PathBuf::from("c/api.ts"));
    assert_eq!(issues[0].line, Some(7));
    edges.reverse();
    assert_eq!(messages(&graph::cycles(&edges)), messages(&issues));
}

#[test]
fn repeated_edges_and_diamond_are_not_cycles() {
    let edges = [
        edge("a/x.rs", "b/x.rs"),
        edge("a/x.rs", "b/x.rs"),
        edge("a/x.rs", "c/x.rs"),
        edge("b/x.rs", "d/x.rs"),
        edge("c/x.rs", "d/x.rs"),
        edge("d/x.rs", "d/y.rs"),
    ];
    assert!(graph::cycles(&edges).is_empty());
}

#[test]
fn cycles_crossing_different_subdirectories_are_visible_at_the_boundary() {
    let edges = [
        edge("src/a/impl/read.rs", "src/b/api/entry.rs"),
        edge("src/b/impl/write.rs", "src/a/api/entry.rs"),
    ];
    let issues = graph::cycles(&edges);
    assert_eq!(
        messages(&issues),
        [
            "directory dependency cycle: src/a -> src/b -> src/a; src/a/impl/read.rs:7 uses src/b/api/entry.rs; src/b/impl/write.rs:7 uses src/a/api/entry.rs"
        ]
    );
    assert_eq!(issues[0].line, Some(7));
}
