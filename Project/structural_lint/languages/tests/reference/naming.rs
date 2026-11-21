use crate::structural_lint::languages::{analyze, tests::DETACHED};
use crate::structural_lint::rule_identities::{Kind, Reference};
use std::path::Path;

// DECISION: D156
fn measured(source: &str, kind: Reference) -> Vec<(usize, String)> {
    let analysis = analyze(Path::new("source.rs"), source, DETACHED).unwrap();
    assert_eq!(analysis.parse_error, None);
    let mut found: Vec<_> = analysis
        .measurements
        .into_iter()
        .filter(|item| item.kind == Kind::Reference(kind))
        .map(|item| (item.line, item.symbol))
        .collect();
    found.sort();
    found
}

#[test]
fn rust_flags_format_strings_with_two_or_more_unnamed_placeholders() {
    let source = "fn render(out: &mut String, name: &str, size: u32) {
    let line = format!(\"{} has {}\", name, size);
    write!(out, \"{}:{:>4}\", name, size);
    std::println!(\"{0} and {0}\", name);
    let named = format!(\"{name} has {size}\");
    let single = format!(\"{}\", name);
    let escaped = format!(\"{{}} {}\", name);
    let mixed = format!(\"{name} has {}\", size);
    let text = concat!(\"{}\", \"{}\");
}
#[cfg(test)]
mod tests {
    fn check() -> String {
        format!(\"{} {}\", 1, 2)
    }
}
";
    assert_eq!(
        measured(source, Reference::PositionalFormat),
        [
            (2, "format".to_string()),
            (3, "write".to_string()),
            (4, "println".to_string()),
        ]
    );
}

#[test]
fn rust_flags_a_struct_literal_field_set_from_another_field_name() {
    let source = "fn swap(x: i32, y: i32, width: u32, other: i32) {
    let crossed = Point { x: y, y: x };
    let size = Size { width, height: width };
    let own = Point { x: x, y };
    let plain = Point { x: other, y: y + 1 };
}
";
    assert_eq!(
        measured(source, Reference::CrossedField),
        [(2, "x:y".to_string()), (3, "height:width".to_string())]
    );
}

// DECISION: D211
#[test]
fn rust_flags_an_index_found_in_one_collection_and_read_from_another() {
    let source = "fn start(layout: &Layout, point: Pos, groups: &[Group], layouts: &mut [Shape], name: &str) {
    let role = [layout.movement, layout.jump, layout.look].iter().position(|rect| rect.contains(point));
    if let Some(role) = role {
        let slot = [MOVEMENT, JUMP, LOOK][role];
    }
    let chosen = groups.iter().position(|group| group.name == name).and_then(|index| layouts.get(index));
    let direct = layouts[groups.iter().position(|group| group.name == name).unwrap()];
    let Some(found) = groups.iter().position(|group| group.name == name) else { return };
    let same = groups[found];
    let other = layouts.get_mut(found);
    if let Some(at) = groups.into_iter().position(|group| group.ready) {
        let shape = layouts[at];
    }
    let skipped = groups.iter().skip(1).position(|group| group.ready).map(|index| layouts[index]);
    let custom = frame.position(name).and_then(|index| layouts.get(index));
    let removed = groups.iter().position(|group| group.ready).map(|index| layouts.swap(index, 0));
}
#[cfg(test)]
mod tests {
    fn check(groups: &[Group], layouts: &[Shape]) {
        let index = groups.iter().position(|group| group.ready).unwrap();
        let shape = &layouts[index];
    }
}
";
    assert_eq!(
        measured(source, Reference::ParallelIndex),
        [
            (4, "position".to_string()),
            (6, "position".to_string()),
            (7, "position".to_string()),
            (10, "position".to_string()),
            (12, "position".to_string()),
        ]
    );
}
