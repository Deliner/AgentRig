use super::measured;
use crate::structural_lint::rule_identities::control_flow::ControlFlow;

#[test]
fn rust_flags_if_chains_and_match_guards_splitting_one_value_by_literal_bounds() {
    let source = "fn grade(score: i32, size: usize) -> &'static str {
    if score < 50 {
        \"low\"
    } else if 80 > score {
        \"mid\"
    } else {
        \"high\"
    }
    match size {
        n if n <= 10 => \"small\",
        n if n <= 100 => \"medium\",
        _ => \"large\",
    }
}
";
    assert_eq!(
        measured(source, ControlFlow::RangeChain),
        [
            (2, "if_expression".to_string()),
            (9, "match_expression".to_string()),
        ]
    );
}

#[test]
fn rust_skips_single_bounds_mixed_subjects_named_limits_justified_and_test_chains() {
    let source = "fn grade(score: i32, other: i32, size: usize) -> i32 {
    if score < 50 {
        return 1;
    }
    if score < 50 {
        1
    } else if other < 80 {
        2
    } else {
        3
    };
    if score < LIMIT {
        1
    } else if score < MAX {
        2
    } else {
        3
    };
    match size {
        n if n <= 10 => 1,
        m if m > 100 => 2,
        _ => 3,
    };
    // DECISION: D001
    if score < 50 { 1 } else if score < 80 { 2 } else { 3 }
}
#[cfg(test)]
mod tests {
    fn check(score: i32) -> i32 {
        if score < 0 { 0 } else if score < 10 { 1 } else { 2 }
    }
}
";
    assert!(measured(source, ControlFlow::RangeChain).is_empty());
}
