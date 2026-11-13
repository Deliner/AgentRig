mod hidden_exit;
mod mixed_junction;
mod range_chain;
mod stale_check;
mod variant_equality;
mod variant_list;
use crate::structural_lint::languages::analyze;
use crate::structural_lint::rule_identities::control_flow::ControlFlow;
use crate::structural_lint::rule_identities::kind::Kind;
use std::path::Path;

// DECISION: D155
fn measured(source: &str, kind: ControlFlow) -> Vec<(usize, String)> {
    let analysis = analyze(Path::new("source.rs"), source, &[]).unwrap();
    assert_eq!(analysis.parse_error, None);
    let mut found: Vec<_> = analysis
        .measurements
        .into_iter()
        .filter(|item| item.kind == Kind::ControlFlow(kind))
        .map(|item| (item.line, item.symbol))
        .collect();
    found.sort();
    found
}

#[test]
fn rust_flags_unlabeled_exits_of_a_loop_nested_in_another_loop_of_the_same_function() {
    let source = "fn scan(rows: &[Vec<u8>]) {
    for row in rows {
        for cell in row {
            if *cell == 0 {
                continue;
            }
            break;
        }
    }
    loop {
        while ready() {
            let value = loop {
                break 5;
            };
        }
        break;
    }
}
";
    assert_eq!(
        measured(source, ControlFlow::UnlabeledExit),
        [
            (5, "continue".to_string()),
            (7, "break".to_string()),
            (13, "break".to_string()),
        ]
    );
}

#[test]
fn rust_skips_labeled_single_level_closure_bounded_justified_and_test_exits() {
    let source = "fn scan(rows: &[Vec<u8>]) {
    for row in rows {
        'cells: for cell in row {
            for other in row {
                continue 'cells;
            }
            // DECISION: D001
            break;
        }
        row.iter().for_each(|cell| loop {
            break;
        });
        break;
    }
}
#[cfg(test)]
mod tests {
    fn check() {
        for a in 0..2 {
            for b in 0..2 {
                break;
            }
        }
    }
}
";
    assert!(measured(source, ControlFlow::UnlabeledExit).is_empty());
}

#[test]
fn rust_flags_subtraction_from_a_length_or_count_call() {
    let source = "fn last(items: &[u8], text: &str) -> usize {
    let index = items.len() - 1;
    let chars = text.chars().count() - 2;
    let head = &items[..items.len() - 1];
    let total = items.len() + 1;
    let gap = 10 - items.len();
    let checked = items.len().saturating_sub(1);
    let matches = text.count(pattern) - 1;
    let plain = len - 1;
    // DECISION: D001
    let justified = items.len() - 1;
    index
}
#[cfg(test)]
mod tests {
    fn check(items: &[u8]) -> usize {
        items.len() - 1
    }
}
";
    assert_eq!(
        measured(source, ControlFlow::LengthSubtraction),
        [
            (2, "len".to_string()),
            (3, "count".to_string()),
            (4, "len".to_string()),
        ]
    );
}
