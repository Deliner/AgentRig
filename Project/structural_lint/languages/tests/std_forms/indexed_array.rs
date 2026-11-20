use super::measured;
use crate::structural_lint::rule_identities::reference::Reference;

#[test]
fn rust_flags_an_array_literal_with_calls_indexed_by_a_computed_value() {
    let source = "fn bounds(values: Values, end: usize) -> Option<u32> {
    [values.clone().min(), values.max()][end]
}
fn pick(step: usize) -> u32 {
    let tail = [first(), second()][1..];
    [[left(), right()], [0, 1]][step][0]
}
#[cfg(test)]
mod tests {
    fn chosen(i: usize) -> u32 {
        [one(), two()][i]
    }
}
";
    let found = measured(source, Reference::IndexedArrayLiteral);
    let lines: Vec<usize> = found.iter().map(|(line, _)| *line).collect();
    assert_eq!(lines, [2, 5, 6, 11]);
    assert!(found.iter().all(|(_, symbol)| symbol == "[..][index]"));
}

#[test]
fn rust_skips_literal_index_repeat_constant_and_whole_range_arrays() {
    let source = "fn chosen(name: &str, rest: &[&str], i: usize) -> u32 {
    let literal = [first(), second()][0];
    let repeated = [make(); 4][i];
    let constant = [u32::MIN, u32::MAX][i];
    let whole = [&[name.trim(), \"x\"][..], rest].concat();
    literal
}
";
    assert!(measured(source, Reference::IndexedArrayLiteral).is_empty());
}
