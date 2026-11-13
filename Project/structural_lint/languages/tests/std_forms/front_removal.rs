use super::measured;
use crate::structural_lint::rule_identities::reference::Reference;

#[test]
fn rust_flags_remove_of_the_first_element_by_a_zero_literal() {
    let source = "fn drain(queue: &mut Vec<u32>, text: &mut String) -> u32 {
    let first = queue.remove(0);
    let letter = text.remove(0x0);
    first
}
";
    assert_eq!(
        measured(source, Reference::FrontRemoval),
        [(2, "remove(0)".into()), (3, "remove(0)".into())]
    );
}

#[test]
fn rust_skips_other_indexes_and_methods_bare_calls_references_justified_and_test_removals() {
    let source = "fn drain(queue: &mut VecDeque<u32>, items: &mut Vec<u32>, map: &mut HashMap<u32, u32>, index: usize) {
    queue.pop_front();
    items.remove(1);
    items.remove(index);
    items.swap_remove(0);
    remove(0);
    map.remove(&0);
    // DECISION: D001
    items.remove(0);
}
#[cfg(test)]
mod tests {
    fn check(items: &mut Vec<u32>) -> u32 {
        items.remove(0)
    }
}
";
    assert!(measured(source, Reference::FrontRemoval).is_empty());
}
