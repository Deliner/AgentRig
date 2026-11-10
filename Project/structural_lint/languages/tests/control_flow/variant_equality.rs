use super::measured;
use crate::structural_lint::rules::control_flow::ControlFlow;

#[test]
fn rust_flags_equality_with_a_variant_path_and_matches_naming_a_variant() {
    let source = "fn pick(mode: Mode, kind: Kind) -> bool {
    let fast = mode == Mode::Fast;
    let other = Status::Done != state;
    let some = matches!(kind, Kind::Leaf | Kind::Branch(_));
    let nested = std::matches!(entry, Some(Entry::File { .. }));
    fast
}
";
    assert_eq!(
        measured(source, ControlFlow::VariantEquality),
        [
            (2, "Mode::Fast".to_string()),
            (3, "Status::Done".to_string()),
            (4, "matches".to_string()),
            (5, "matches".to_string()),
        ]
    );
}

#[test]
fn rust_skips_constants_calls_literal_patterns_justified_and_test_comparisons() {
    let source = "fn pick(value: i32, name: &str, mode: Option<Mode>) -> bool {
    let limit = value == i32::MAX;
    let constant = value == Self::DEFAULT_SIZE;
    let single = value == Fast;
    let wrapped = mode == Some(Mode::Fast);
    let text = matches!(name, \"a\" | \"b\");
    let range = matches!(value, 1..=3);
    let bound = matches!(mode, Some(m) if m.ready());
    // DECISION: D001
    let justified = mode == Mode::Fast;
    limit
}
#[cfg(test)]
mod tests {
    fn check(mode: Mode) -> bool {
        mode == Mode::Fast
    }
}
";
    assert!(measured(source, ControlFlow::VariantEquality).is_empty());
}
