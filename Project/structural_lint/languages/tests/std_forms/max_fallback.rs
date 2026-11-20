use super::measured;
use crate::structural_lint::rule_identities::Reference;

// DECISION: D209
#[test]
fn rust_flags_a_max_fallback_added_to_or_multiplied_on_either_side() {
    let source = "fn bytes(queue: &Queue, bulk: &Bulk, cursor: u64, limit: Option<u32>) -> usize {
    let total = usize::try_from(queue.bytes()).unwrap_or(usize::MAX) + bulk.bytes();
    let x = i64::try_from(cursor % 64 / 8).unwrap_or(i64::MAX) * 4 + 1;
    let after = 1 + limit.unwrap_or(std::u32::MAX);
    let step = steps.get(0).copied().unwrap_or(MAX) * 2;
    total
}
";
    assert_eq!(
        measured(source, Reference::MaxFallback),
        [
            (2, "MAX".into()),
            (3, "MAX".into()),
            (4, "MAX".into()),
            (5, "MAX".into()),
        ]
    );
}

// DECISION: D209
#[test]
fn rust_skips_saturating_subtracted_compared_other_lazy_and_test_fallbacks() {
    let source = "fn bounded(limit: Option<u32>) -> u32 {
    let saturated = limit.unwrap_or(u32::MAX).saturating_add(1);
    let below = 10 - limit.unwrap_or(u32::MAX);
    let compared = limit.unwrap_or(u32::MAX) > 3;
    let minimum = limit.unwrap_or(u32::MIN) + 1;
    let other = limit.unwrap_or(LIMIT) * 2;
    let lazy = limit.unwrap_or_else(|| u32::MAX) + 1;
    let alone = limit.unwrap_or(u32::MAX);
    saturated
}
#[cfg(test)]
mod tests {
    fn check(limit: Option<u32>) -> u32 {
        limit.unwrap_or(u32::MAX) * 2
    }
}
";
    assert!(measured(source, Reference::MaxFallback).is_empty());
}
