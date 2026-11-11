use super::measured;
use crate::structural_lint::rules::kind::Kind;
use crate::structural_lint::rules::signature::Signature;

const ALIAS: Kind = Kind::Signature(Signature::PrimitiveAlias);
const OPTION: Kind = Kind::Signature(Signature::AmbiguousOption);

#[test]
fn rust_flags_aliases_of_a_scalar_or_string_but_not_generic_or_other_types() {
    let source = "type Meters = f64;
pub type UserId = u64;
type Name = String;
type Flag = bool;
type Pair<T> = (T, T);
type Wide<T> = u64;
type Store = HashMap<String, u64>;
type Text = std::string::String;
// DECISION: D001
type Seconds = u32;
";
    assert_eq!(
        measured(source, ALIAS),
        [
            (1, "Meters".to_string()),
            (2, "UserId".to_string()),
            (3, "Name".to_string()),
        ]
    );
}

// DECISION: D162
#[test]
fn rust_flags_an_option_over_a_bool_wherever_it_stands_and_leaves_a_nested_option_to_clippy() {
    let source = "struct Settings {
    verbose: Option<bool>,
    timeout: Option<Option<u32>>,
    limit: Option<u32>,
}
fn choose(flag: std::option::Option<bool>) -> Option<Vec<bool>> {
    None
}
// DECISION: D001
type Tristate = Option<bool>;
";
    assert_eq!(
        measured(source, OPTION),
        [
            (2, "Option<bool>".to_string()),
            (6, "std::option::Option<bool>".to_string()),
        ]
    );
}
