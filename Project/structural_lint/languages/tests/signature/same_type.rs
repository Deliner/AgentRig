use super::measured;
use crate::structural_lint::rules::kind::Kind;
use crate::structural_lint::rules::signature::Signature;

const PARAMETERS: Kind = Kind::Signature(Signature::SameTypeParameters);
const POSITIONS: Kind = Kind::Signature(Signature::SameTypePositions);
const FLAG: Kind = Kind::Signature(Signature::PositionalFlag);

#[test]
fn rust_flags_each_function_with_two_parameters_of_one_type_once() {
    let source = "fn copy(from: &Path, to: &Path, mode: u32, retries: u32) {}
trait Store {
    fn put(&self, key: String, value: String);
}
impl Grid {
    fn cell(&self, row: usize, column: usize) -> u8 {
        0
    }
}
";
    assert_eq!(
        measured(source, PARAMETERS),
        [
            (1, "copy".to_string()),
            (3, "put".to_string()),
            (6, "cell".to_string()),
        ]
    );
}

#[test]
fn rust_skips_distinct_types_receivers_trait_impls_justified_and_test_functions() {
    let source = "fn open(path: &Path, mode: Mode) {}
impl Grid {
    fn merge(&self, other: &Self) {}
}
impl Add for Meters {
    fn add(self, left: Meters, right: Meters) -> Meters {
        left
    }
}
// DECISION: D001
fn span(start: usize, end: usize) {}
#[cfg(test)]
mod tests {
    fn check(left: u32, right: u32) {}
}
";
    assert!(measured(source, PARAMETERS).is_empty());
}

#[test]
fn rust_flags_tuple_structs_and_variants_with_two_positions_of_one_type() {
    let source = "struct Span(usize, usize);
struct Label(String, u32);
enum Shape {
    Circle(f64),
    Rectangle(f64, f64),
    Named { width: f64, height: f64 },
}
fn bounds() -> (usize, usize) {
    (0, 1)
}
// DECISION: D001
struct Pair(u8, u8);
";
    assert_eq!(
        measured(source, POSITIONS),
        [(1, "Span".to_string()), (5, "Rectangle".to_string())]
    );
}

// DECISION: D166
#[test]
fn rust_flags_tuple_structs_and_variants_with_a_bool_among_two_or_more_positions() {
    let source = "pub struct Save(pub PathBuf, pub bool);
struct Verbose(bool);
struct Twin(bool, bool);
enum Command {
    Toggle(u32, bool),
    Quiet(bool),
    Named { path: PathBuf, force: bool },
    Wrapped(Option<bool>, u32),
}
struct Plain(String, u32);
// DECISION: D001
struct Justified(u8, bool);
#[cfg(test)]
mod tests {
    struct Probe(u8, bool);
}
";
    assert_eq!(
        measured(source, FLAG),
        [
            (1, "Save".to_string()),
            (3, "Twin".to_string()),
            (5, "Toggle".to_string()),
        ]
    );
}
