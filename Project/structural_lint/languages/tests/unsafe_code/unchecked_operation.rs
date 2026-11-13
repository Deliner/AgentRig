use super::measured;
use crate::structural_lint::rule_identities::reference::Reference;

#[test]
fn rust_flags_transmute_paths_and_unchecked_calls() {
    let source =
        "fn convert(bits: u32, bytes: [u8; 4], text: &[u8], items: &[u8], index: usize) -> f32 {
    let value: f32 = unsafe { std::mem::transmute(bits) };
    let copied = unsafe { mem::transmute_copy::<[u8; 4], u32>(&bytes) };
    let typed = unsafe { transmute::<u32, f32>(bits) };
    let decoded = unsafe { std::str::from_utf8_unchecked(text) };
    let item = unsafe { items.get_unchecked(index) };
    let sum = unsafe { index.unchecked_add(index) };
    let next = unsafe { cursor.next_unchecked() };
    value
}
";
    assert_eq!(
        measured(source, Reference::TransmuteCall),
        [
            (2, "transmute".into()),
            (3, "transmute_copy".into()),
            (4, "transmute".into()),
        ]
    );
    assert_eq!(
        measured(source, Reference::UncheckedCall),
        [
            (5, "from_utf8_unchecked".into()),
            (6, "get_unchecked".into()),
            (7, "unchecked_add".into()),
            (8, "next_unchecked".into()),
        ]
    );
}

#[test]
fn rust_skips_checked_literal_method_inherited_justified_and_test_operations() {
    let source = "fn convert(value: u32, items: &[u8]) -> u8 {
    let checked = items.get(0);
    let literal = unsafe { NonZeroU32::new_unchecked(1) };
    let method = value.transmute();
    // DECISION: D001
    let justified = unsafe { items.get_unchecked(value as usize) };
    // DECISION: D001
    let bits: u32 = unsafe { std::mem::transmute(1.0f32) };
    0
}
unsafe fn read_unchecked(items: &[u8], index: usize) -> u8 {
    unsafe { *items.get_unchecked(index) }
}
#[cfg(test)]
mod tests {
    fn check(bits: u32, items: &[u8]) -> f32 {
        unsafe { items.get_unchecked(bits as usize) };
        unsafe { std::mem::transmute(bits) }
    }
}
";
    assert!(measured(source, Reference::TransmuteCall).is_empty());
    assert!(measured(source, Reference::UncheckedCall).is_empty());
}
