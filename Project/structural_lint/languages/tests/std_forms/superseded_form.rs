use crate::structural_lint::languages::analyze;
use crate::structural_lint::rule_identities::kind::Kind;
use crate::structural_lint::rule_identities::reference::Reference;
use std::path::Path;

fn superseded(source: &str) -> Vec<(usize, String)> {
    let analysis = analyze(Path::new("source.rs"), source, &[]).unwrap();
    assert_eq!(analysis.parse_error, None);
    let mut found: Vec<_> = analysis
        .measurements
        .into_iter()
        .filter(|item| item.kind == Kind::Reference(Reference::SupersededForm))
        .map(|item| (item.line, item.symbol))
        .collect();
    found.sort();
    found
}

// DECISION: D177
#[test]
fn rust_names_the_current_form_of_each_superseded_form_in_product_and_test_code() {
    let source =
        "fn shown(path: &Path, root: &Path, name: String, file: &str) -> Result<(), Infallible> {
    let bare = file.strip_prefix(\"./\").unwrap_or(file);
    let stem = name.strip_suffix(\".exe\").unwrap_or(&name);
    let relative = path
        .strip_prefix(root)
        .unwrap_or(path);
    let squares = (0..4).map(|index| index * index).collect::<Vec<_>>();
    let names: Vec<String> = (0usize..count).map(label).collect();
    Ok(())
}
fn never(value: std::convert::Infallible) {}
#[cfg(test)]
mod tests {
    fn sample() -> Vec<usize> {
        (0..3).map(|index| index + 1).collect::<std::vec::Vec<usize>>()
    }
}
";
    let trim_prefix = "strip_prefix(p).unwrap_or(x) -> trim_prefix(p)";
    let trim_suffix = "strip_suffix(p).unwrap_or(x) -> trim_suffix(p)";
    let from_fn = "(0..n).map(f).collect() -> Vec::from_fn(n, f)";
    let never = "Infallible -> !";
    assert_eq!(
        superseded(source),
        [
            (1, never.to_string()),
            (2, trim_prefix.to_string()),
            (3, trim_suffix.to_string()),
            (4, trim_prefix.to_string()),
            (7, from_fn.to_string()),
            (8, from_fn.to_string()),
            (11, never.to_string()),
            (15, from_fn.to_string()),
        ]
    );
}

#[test]
fn rust_accepts_current_forms_other_fallbacks_ranges_targets_and_justified_lines() {
    let source = "fn current(path: &Path, root: &Path, file: &str, text: &str) -> Result<(), !> {
    let bare = file.trim_prefix(\"./\");
    let relative = path.trim_prefix(root);
    let squares = Vec::from_fn(4, |index| index * index);
    let key = flag.strip_prefix(\"--\").unwrap_or(\"\");
    let other = file.strip_prefix(\"./\").unwrap_or(fallback);
    let shifted = (1..4).map(|index| index * 2).collect::<Vec<_>>();
    let closed = (0..=4).map(|index| index * 2).collect::<Vec<_>>();
    let unique = (0..4).map(|index| index % 2).collect::<HashSet<_>>();
    let inferred = (0..4).map(|index| index * 2).collect();
    let narrow = (0u32..4).map(|index| index * 2).collect::<Vec<_>>();
    // DECISION: D177
    let kept = text.strip_suffix(\"/\").unwrap_or(text);
    Ok(())
}
";
    assert_eq!(superseded(source), []);
}

// DECISION: D178
#[test]
fn rust_names_vec_from_fn_for_an_index_collect_returned_as_a_vec() {
    let source = "fn labels(count: usize) -> Vec<String> {
    (0..count).map(label).collect()
}
fn early(count: usize) -> std::vec::Vec<usize> {
    if count == 0 {
        return (0..count).map(|index| index + 1).collect();
    }
    Vec::new()
}
fn wide() {
    let edges = |count: usize| -> Vec<Edge> {
        (0..count)
            .map(|index| edge(index))
            .collect()
    };
}
";
    let from_fn = "(0..n).map(f).collect() -> Vec::from_fn(n, f)";
    assert_eq!(
        superseded(source),
        [
            (2, from_fn.to_string()),
            (6, from_fn.to_string()),
            (12, from_fn.to_string()),
        ]
    );
}

#[test]
fn rust_accepts_returned_collects_into_other_types_branches_inner_closures_and_justified_lines() {
    let source = "fn labels(count: usize) -> Vec<String> {
    Vec::from_fn(count, label)
}
fn unique(count: usize) -> HashSet<usize> {
    (0..count).map(|index| index % 2).collect()
}
fn parsed(count: usize) -> Option<Vec<usize>> {
    (0..count).map(|index| index.checked_add(1)).collect()
}
fn branch(flag: bool) -> Vec<usize> {
    if flag {
        (0..2).map(|index| index).collect()
    } else {
        Vec::new()
    }
}
fn outer() -> Vec<usize> {
    let inner = |count: usize| {
        return (0..count).map(|index| index).collect();
    };
    let untyped = |count: usize| (0..count).map(|index| index).collect();
    Vec::new()
}
fn kept(count: usize) -> Vec<u32> {
    // DECISION: D178
    (0..count).map(|index| index as u32).collect()
}
";
    assert_eq!(superseded(source), []);
}
