pub(super) fn supported(arguments: &str) -> bool {
    arguments
        .strip_prefix('(')
        .and_then(|text| text.strip_suffix(')'))
        .is_some_and(predicate)
}
fn predicate(text: &str) -> bool {
    if let Some((operator, rest)) = text.split_once('(') {
        return combination(operator, rest);
    }
    if let Some((key, value)) = text.split_once('=') {
        let target = matches!(key, "target_os" | "target_family");
        return target && literal(value);
    }
    matches!(text, "test" | "unix" | "windows")
}
fn combination(operator: &str, rest: &str) -> bool {
    let arguments = rest.strip_suffix(')').and_then(split);
    match (operator, arguments) {
        ("not", Some(arguments)) => arguments.len() == 1 && arguments.iter().all(|a| predicate(a)),
        ("any" | "all", Some(arguments)) => arguments.iter().all(|a| predicate(a)),
        _ => false,
    }
}
fn literal(value: &str) -> bool {
    value
        .strip_prefix('"')
        .and_then(|value| value.strip_suffix('"'))
        .is_some_and(|name| {
            !name.is_empty()
                && name
                    .chars()
                    .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
        })
}
fn split(text: &str) -> Option<Vec<&str>> {
    let mut depth = 0usize;
    let mut start = 0;
    let mut parts = Vec::new();
    for (index, character) in text.char_indices() {
        match character {
            '(' => depth += 1,
            ')' => depth = depth.checked_sub(1)?,
            ',' if depth == 0 => {
                parts.push(&text[start..index]);
                start = index + 1;
            }
            _ => {}
        }
    }
    parts.push(&text[start..]);
    let balanced = depth == 0;
    balanced.then_some(parts)
}

#[cfg(test)]
mod tests {
    use super::supported;

    #[test]
    fn platform_and_test_predicates_are_supported() {
        for condition in [
            "(test)",
            "(unix)",
            "(target_os=\"macos\")",
            "(target_family=\"unix\")",
            "(not(target_os=\"linux\"))",
            "(all(unix,not(any(target_os=\"macos\",target_os=\"ios\"))))",
        ] {
            assert!(supported(condition), "{condition}");
        }
    }

    #[test]
    fn feature_unknown_and_malformed_predicates_are_unsupported() {
        for condition in [
            "(feature=\"extra\")",
            "(debug_assertions)",
            "(not(unix,windows))",
            "(any(unix,feature=\"extra\"))",
            "(all(unix,))",
            "(target_os=linux)",
            "(target_os=\"Linux\")",
            "(all(unix)",
            "test",
        ] {
            assert!(!supported(condition), "{condition}");
        }
    }
}
