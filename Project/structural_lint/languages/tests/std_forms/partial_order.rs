use super::measured;
use crate::structural_lint::rule_identities::reference::Reference;

#[test]
fn rust_flags_a_fixed_ordering_in_place_of_a_missing_partial_comparison() {
    let source = "fn order(values: &mut [f64], a: f64, b: f64) -> Ordering {
    values.sort_by(|x, y| x.partial_cmp(y).unwrap_or(Ordering::Equal));
    let reversed = b.partial_cmp(&a).unwrap_or_else(|| Ordering::Less);
    a.partial_cmp(&b).unwrap_or(Greater)
}
";
    assert_eq!(
        measured(source, Reference::PartialOrderFallback),
        [
            (2, "unwrap_or".into()),
            (3, "unwrap_or_else".into()),
            (4, "unwrap_or".into()),
        ]
    );
}

#[test]
fn rust_skips_total_order_variable_fallbacks_mapped_other_justified_and_test_comparisons() {
    let source = "fn order(a: f64, b: f64, fallback: Ordering) -> Ordering {
    let total = a.total_cmp(&b);
    let passed = a.partial_cmp(&b).unwrap_or(fallback);
    let other = a.compare(&b).unwrap_or(Ordering::Equal);
    let mapped = a.partial_cmp(&b).map(Ordering::reverse).unwrap_or(Ordering::Equal);
    // DECISION: D001
    let justified = a.partial_cmp(&b).unwrap_or(Ordering::Equal);
    total
}
#[cfg(test)]
mod tests {
    fn check(a: f64, b: f64) -> Ordering {
        a.partial_cmp(&b).unwrap_or(Ordering::Equal)
    }
}
";
    assert!(measured(source, Reference::PartialOrderFallback).is_empty());
}
