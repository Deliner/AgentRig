use super::measured;
use crate::structural_lint::rule_identities::reference::Reference;

#[test]
fn rust_flags_default_hasher_type_names_and_unstable_sorts_by_a_comparator_or_key() {
    let source = "use std::collections::hash_map::DefaultHasher;
fn key(value: &str, items: &mut [(u32, String)]) -> u64 {
    let mut hasher = DefaultHasher::new();
    let other = std::hash::DefaultHasher::default();
    let name = std::any::type_name::<String>();
    let shown = type_name_of_val(&value);
    items.sort_unstable_by_key(|item| item.0);
    items.sort_unstable_by(|a, b| a.0.cmp(&b.0));
    hasher.finish()
}
";
    assert_eq!(
        measured(source, Reference::UnstableValue),
        [
            (3, "DefaultHasher".into()),
            (4, "DefaultHasher".into()),
            (5, "type_name".into()),
            (6, "type_name_of_val".into()),
        ]
    );
    assert_eq!(
        measured(source, Reference::UnstableSort),
        [
            (7, "sort_unstable_by_key".into()),
            (8, "sort_unstable_by".into()),
        ]
    );
}

#[test]
fn rust_skips_hasher_types_and_imports_named_calls_other_sorts_justified_and_test_code() {
    let source = "use std::hash::DefaultHasher;
fn key(hasher: &mut DefaultHasher, items: &mut [u32], value: &str) -> &'static str {
    let state: BuildHasherDefault<DefaultHasher> = BuildHasherDefault::default();
    items.sort_unstable();
    items.sort_by_key(|item| *item);
    let named = type_name(value);
    // DECISION: D001
    let justified = std::any::type_name::<u32>();
    named
}
#[cfg(test)]
mod tests {
    fn check(items: &mut [u32]) -> &'static str {
        items.sort_unstable_by_key(|item| *item);
        std::any::type_name::<u32>()
    }
}
";
    assert!(measured(source, Reference::UnstableValue).is_empty());
    assert!(measured(source, Reference::UnstableSort).is_empty());
}
