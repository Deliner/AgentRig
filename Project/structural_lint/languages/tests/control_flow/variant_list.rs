use super::measured;
use crate::structural_lint::rule_identities::control_flow::ControlFlow;

#[test]
fn rust_flags_arrays_listing_every_unit_variant_of_a_same_file_enum_in_order() {
    let source = "enum Color {
    Red,
    Green,
    Blue,
    Custom(u8),
}
const ALL: [Color; 3] = [Color::Red, Color::Green, Color::Blue];
impl Color {
    fn all() -> &'static [Self] {
        &[Self::Red, Self::Green, Self::Blue]
    }
}
fn each() {
    for color in [Color::Red, Color::Green, Color::Blue] {}
}
";
    assert_eq!(
        measured(source, ControlFlow::VariantList),
        [
            (7, "Color".into()),
            (10, "Color".into()),
            (14, "Color".into()),
        ]
    );
}

#[test]
fn rust_skips_partial_reordered_short_foreign_mixed_repeated_justified_and_test_lists() {
    let source = "enum Mode {
    Fast,
    Slow,
    Idle,
    Off,
}
enum Pair {
    Left,
    Right,
}
const SOME: [Mode; 3] = [Mode::Fast, Mode::Slow, Mode::Idle];
const SHUFFLED: [Mode; 4] = [Mode::Slow, Mode::Fast, Mode::Idle, Mode::Off];
const BOTH: [Pair; 2] = [Pair::Left, Pair::Right];
const FOREIGN: [Other; 3] = [Other::A, Other::B, Other::C];
const MIXED: [Mode; 4] = [Mode::Fast, Mode::Slow, Mode::Idle, Pair::Left];
const REPEATED: [Mode; 3] = [Mode::Fast; 3];
// DECISION: D001
const JUSTIFIED: [Mode; 4] = [Mode::Fast, Mode::Slow, Mode::Idle, Mode::Off];
#[cfg(test)]
mod tests {
    const ALL: [Mode; 4] = [Mode::Fast, Mode::Slow, Mode::Idle, Mode::Off];
}
";
    assert!(measured(source, ControlFlow::VariantList).is_empty());
}
