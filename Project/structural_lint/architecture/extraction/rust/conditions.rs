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
