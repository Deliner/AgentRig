use super::measured;
use crate::structural_lint::rule_identities::reference::Reference;

#[test]
fn rust_flags_float_epsilon_compared_directly_alone_in_parentheses_or_negated() {
    let source = "fn close(a: f64, b: f32) -> bool {
    let near = (a - 1.0).abs() < f64::EPSILON;
    let small = b.abs() <= std::f32::EPSILON;
    let above = a > -(f64::EPSILON);
    let exact = f64::EPSILON == a;
    near
}
";
    assert_eq!(
        measured(source, Reference::EpsilonTolerance),
        [
            (2, "EPSILON".into()),
            (3, "EPSILON".into()),
            (4, "EPSILON".into()),
            (5, "EPSILON".into()),
        ]
    );
}

#[test]
fn rust_skips_scaled_unequal_named_other_constant_justified_and_test_tolerances() {
    let source = "fn scaled(a: f64, b: f64, tolerance: f64) -> bool {
    let relative = (a - b).abs() < f64::EPSILON * a.abs();
    let unequal = a != f64::EPSILON;
    let named = (a - b).abs() < tolerance;
    let other = a < f64::MAX;
    let single = a < EPSILON;
    // DECISION: D001
    let justified = a < f64::EPSILON;
    relative
}
#[cfg(test)]
mod tests {
    fn check(a: f64) -> bool {
        a < f64::EPSILON
    }
}
";
    assert!(measured(source, Reference::EpsilonTolerance).is_empty());
}
