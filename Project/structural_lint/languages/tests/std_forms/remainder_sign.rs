use super::measured;
use crate::structural_lint::rule_identities::reference::Reference;

#[test]
fn rust_flags_a_remainder_equal_to_a_nonzero_literal_or_ordered_against_any_literal() {
    let source = "fn odd(value: i32, step: f64) -> bool {
    let odd = value % 2 == 1;
    let other = (value % 4) != 3;
    let low = value % 10 < 5;
    let flipped = 0 <= value % 3;
    let fraction = step % 1.0 > 0.5;
    odd
}
";
    assert_eq!(
        measured(source, Reference::RemainderSign),
        [
            (2, "%".into()),
            (3, "%".into()),
            (4, "%".into()),
            (5, "%".into()),
            (6, "%".into()),
        ]
    );
}

#[test]
fn rust_skips_zero_equality_named_bounds_negated_literals_justified_and_test_remainders() {
    let source = "fn even(value: i32, limit: i32) -> bool {
    let even = value % 2 == 0;
    let odd = value % 2 != 0;
    let bounded = value % 10 < limit;
    let negative = value % 2 == -1;
    let sum = value + 2 == 1;
    let both = value % 2 == limit % 2;
    // DECISION: D001
    let justified = value % 2 == 1;
    even
}
#[cfg(test)]
mod tests {
    fn check(value: i32) -> bool {
        value % 2 == 1
    }
}
";
    assert!(measured(source, Reference::RemainderSign).is_empty());
}
