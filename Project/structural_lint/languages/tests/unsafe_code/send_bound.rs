use super::measured;
use crate::structural_lint::rule_identities::reference::Reference;

#[test]
fn rust_flags_unsafe_send_and_sync_impls_with_an_unbounded_type_parameter() {
    let source = "struct Shared<T>(*mut T);
unsafe impl<T> Send for Shared<T> {}
unsafe impl<T: Clone> Sync for Shared<T> {}
unsafe impl<T, U: Send> std::marker::Send for Pair<T, U> {}
unsafe impl<T> Sync for Cell<T> where T: Clone {}
";
    assert_eq!(
        measured(source, Reference::UnboundedSend),
        [
            (2, "Send".into()),
            (3, "Sync".into()),
            (4, "Send".into()),
            (5, "Sync".into()),
        ]
    );
}

#[test]
fn rust_skips_bounded_concrete_safe_other_trait_justified_and_test_impls() {
    let source = "unsafe impl<T: Send> Send for Shared<T> {}
unsafe impl<T> Sync for Shared<T> where T: Sync {}
unsafe impl<'a, T: ?Sized + std::marker::Sync> Sync for View<'a, T> {}
unsafe impl Send for Handle {}
impl<T> Clone for Shared<T> {}
unsafe impl<T> Allocator for Shared<T> {}
// DECISION: D001
unsafe impl<T> Send for Other<T> {}
#[cfg(test)]
mod tests {
    unsafe impl<T> Send for Probe<T> {}
}
";
    assert!(measured(source, Reference::UnboundedSend).is_empty());
}
