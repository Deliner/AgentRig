use crate::structural_lint::languages::analyze;
use crate::structural_lint::rules::kind::Kind;
use crate::structural_lint::rules::signature::Signature;
use std::path::Path;

// DECISION: D119
fn mismatches(source: &str) -> Vec<(usize, String)> {
    let analysis = analyze(Path::new("source.rs"), source).unwrap();
    assert_eq!(analysis.parse_error, None);
    let mut found: Vec<_> = analysis
        .measurements
        .into_iter()
        .filter(|item| item.kind == Kind::Signature(Signature::OrdEqMismatch))
        .map(|item| (item.line, item.symbol))
        .collect();
    found.sort();
    found
}

#[test]
fn rust_flags_order_reading_fewer_or_more_fields_than_equality() {
    let source = "#[derive(Debug, PartialEq, Eq)]
struct Dependency {
    name: String,
    version: u32,
    path: String,
}
impl Ord for Dependency {
    fn cmp(&self, other: &Self) -> Ordering {
        self.name.cmp(&other.name)
    }
}
impl PartialOrd for Dependency {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
struct Rank {
    id: u32,
    score: u32,
}
impl PartialEq for Rank {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}
impl PartialOrd for Rank {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.score.cmp(&other.score))
    }
}
";
    let dependency = (7, "Dependency: equality-only path, version".into());
    let rank = (26, "Rank: equality-only id; order-only score".into());
    assert_eq!(mismatches(source), [dependency, rank]);
}

#[test]
fn rust_flags_a_cached_field_reached_through_a_same_file_key_method() {
    let source = "#[derive(PartialEq)]
/// Interned key with its cached hash.
#[repr(C)]
struct Interned<T> {
    key: T,
    hash: u64,
}
impl<T: Ord> std::cmp::Ord for Interned<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.key().cmp(other.key())
    }
}
impl<T> Interned<T> {
    fn key(&self) -> &T {
        &self.key
    }
}
";
    assert_eq!(
        mismatches(source),
        [(8, "Interned: equality-only hash".into())]
    );
}

#[test]
fn rust_accepts_both_relations_through_one_key_method() {
    let source = "struct Version {
    major: u32,
    minor: u32,
}
impl Version {
    fn key(&self) -> (u32, u32) {
        (self.major, self.minor)
    }
}
impl PartialEq for Version {
    fn eq(&self, other: &Self) -> bool {
        self.key() == other.key()
    }
}
impl Ord for Version {
    fn cmp(&self, other: &Self) -> Ordering {
        self.key().cmp(&other.key())
    }
}
impl PartialOrd for Version {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
";
    assert_eq!(mismatches(source), []);
}

#[test]
fn rust_accepts_derived_pairs_equal_field_sets_and_other_right_hand_types() {
    let source = "#[derive(PartialEq, Eq, PartialOrd, Ord)]
struct Derived {
    left: u32,
}
#[derive(PartialEq, Eq)]
struct Point {
    x: i32,
    y: i32,
}
impl Ord for Point {
    fn cmp(&self, other: &Self) -> Ordering {
        (self.x, self.y).cmp(&(other.x, other.y))
    }
}
impl PartialOrd<i32> for Point {
    fn partial_cmp(&self, other: &i32) -> Option<Ordering> {
        self.x.partial_cmp(other)
    }
}
struct Digit {
    value: u32,
}
impl PartialEq for Digit {
    fn eq(&self, other: &Self) -> bool {
        self.value % 10 == other.value % 10
    }
}
impl PartialOrd for Digit {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.value.partial_cmp(&other.value)
    }
}
";
    assert_eq!(mismatches(source), []);
}

#[test]
fn rust_accepts_justified_and_unresolvable_comparisons() {
    let source = "#[derive(PartialEq)]
struct Entry {
    name: String,
    cache: u64,
}
// DECISION: D001
impl Ord for Entry {
    fn cmp(&self, other: &Self) -> Ordering {
        self.name.cmp(&other.name)
    }
}
#[derive(PartialEq)]
struct Remote {
    id: u32,
    label: String,
}
impl Ord for Remote {
    fn cmp(&self, other: &Self) -> Ordering {
        self.sort_key().cmp(&other.sort_key())
    }
}
#[derive(PartialEq)]
struct Tagged {
    tag: u8,
    note: String,
}
impl Ord for Tagged {
    fn cmp(&self, other: &Self) -> Ordering {
        compare(self, other)
    }
}
";
    assert_eq!(mismatches(source), []);
}

#[test]
fn rust_skips_tuple_structs_foreign_types_and_test_code() {
    let source = "#[derive(PartialEq)]
struct Meters(u32, u32);
impl Ord for Meters {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.cmp(&other.0)
    }
}
impl Ord for Foreign {
    fn cmp(&self, other: &Self) -> Ordering {
        self.id.cmp(&other.id)
    }
}
#[cfg(test)]
mod tests {
    #[derive(PartialEq)]
    struct Sample {
        first: u32,
        second: u32,
    }
    impl Ord for Sample {
        fn cmp(&self, other: &Self) -> Ordering {
            self.first.cmp(&other.first)
        }
    }
}
";
    assert_eq!(mismatches(source), []);
}
