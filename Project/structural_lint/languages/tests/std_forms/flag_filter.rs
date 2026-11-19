use super::measured;
use crate::structural_lint::rule_identities::reference::Reference;

// DECISION: D208
#[test]
fn rust_flags_a_filter_by_a_known_flag_on_a_computed_value() {
    let source = "fn save(&mut self, idle: bool, ready: bool, state: &State) -> Option<Save> {
    let next = self.storage.next_save().filter(|_| idle);
    let single = one(&table.uniform).filter(|_| !state.sparse);
    let kept = self.measured.as_mut().filter(|_| ready);
    let skipped = order.iter().filter(|_| !together).count();
    let loaded = load()?.filter(|_| fresh);
    let sized = slots.get(index).filter(|_| sizes[index] > 0);
    next
}
#[cfg(test)]
mod tests {
    fn probe(open: bool) -> usize {
        sky().filter(|_| open).count()
    }
}
";
    let found = measured(source, Reference::OuterFlagFilter);
    let lines: Vec<usize> = found.iter().map(|(line, _)| *line).collect();
    assert_eq!(lines, [2, 3, 4, 5, 6, 7, 13]);
    assert!(found.iter().all(|(_, symbol)| symbol == "filter"));
}

// DECISION: D208
#[test]
fn rust_skips_uncomputed_receivers_used_values_computed_flags_and_justified_filters() {
    let source = "fn keep(&self, flag: bool, field: Option<u32>) -> Option<u32> {
    let plain = field.filter(|_| flag);
    let owned = self.cached.filter(|_| flag);
    let ranged = (0..6).filter(|_| flag).count();
    let valued = lookup().filter(|value| *value > 0);
    let checked = lookup().filter(|_| ready(state));
    let function = lookup().filter(positive);
    let first = flag.then(|| lookup()).flatten();
    // DECISION: D001
    let justified = lookup().filter(|_| flag);
    plain
}
";
    assert!(measured(source, Reference::OuterFlagFilter).is_empty());
}
