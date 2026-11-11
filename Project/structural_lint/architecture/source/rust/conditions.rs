pub(super) fn supported(arguments: &str) -> bool {
    arguments
        .strip_prefix('(')
        .and_then(|text| text.strip_suffix(')'))
        .is_some_and(predicate)
}
fn predicate(text: &str) -> bool {
    if let Some((operator, rest)) = text.split_once('(') {
        let arguments = rest.strip_suffix(')').and_then(split);
        return match (operator, arguments) {
            ("not", Some(arguments)) => {
                arguments.len() == 1 && arguments.iter().all(|a| predicate(a))
            }
            ("any" | "all", Some(arguments)) => arguments.iter().all(|a| predicate(a)),
            _ => false,
        };
    }
    if let Some((key, value)) = text.split_once('=') {
        return match key {
            "target_os" | "target_family" => literal(value, platform_character),
            "feature" => literal(value, feature_character),
            _ => false,
        };
    }
    matches!(text, "test" | "unix" | "windows")
}
fn platform_character(c: char) -> bool {
    c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_'
}
fn feature_character(c: char) -> bool {
    c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '+' | '.')
}
fn literal(value: &str, allowed: fn(char) -> bool) -> bool {
    value
        .strip_prefix('"')
        .and_then(|value| value.strip_suffix('"'))
        .is_some_and(|name| !name.is_empty() && name.chars().all(allowed))
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
                parts.push(text.get(start..index)?);
                start = index + 1;
            }
            _ => {}
        }
    }
    parts.push(text.get(start..)?);
    let balanced = depth == 0;
    balanced.then_some(parts)
}

#[cfg(test)]
mod tests {
    use super::supported;

    #[test]
    fn platform_feature_and_test_predicates_are_supported() {
        for condition in [
            "(test)",
            "(unix)",
            "(target_os=\"macos\")",
            "(target_family=\"unix\")",
            "(not(target_os=\"linux\"))",
            "(all(unix,not(any(target_os=\"macos\",target_os=\"ios\"))))",
            "(feature=\"extra\")",
            "(feature=\"serde-json\")",
            "(any(unix,feature=\"extra\"))",
            "(not(feature=\"Std_2.x+alloc\"))",
            "(all(feature=\"native\",not(target_family=\"wasm\")))",
        ] {
            assert!(supported(condition), "{condition}");
        }
    }

    #[test]
    fn unknown_and_malformed_predicates_are_unsupported() {
        for condition in [
            "(debug_assertions)",
            "(target_arch=\"x86_64\")",
            "(feature=extra)",
            "(feature=\"\")",
            "(feature=\"a b\")",
            "(not(unix,windows))",
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
