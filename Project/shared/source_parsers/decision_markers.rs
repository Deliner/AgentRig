use anyhow::Result;
use std::path::Path;
use tree_sitter::Node;

struct Marker<'a> {
    id: &'a str,
    rule: Option<&'a str>,
}
// DECISION: D219
pub struct Exception {
    pub line: usize,
    pub decision: String,
    pub rule: String,
}

// DECISION: D037
pub fn reference(comment: &str) -> Option<&str> {
    marker(comment).map(|marker| marker.id)
}
// DECISION: D219
pub fn exceptions(path: &Path, source: &str) -> Result<Vec<Exception>> {
    let lines = path
        .extension()
        .is_some_and(|extension| extension == "yaml" || extension == "toml");
    if lines {
        return Ok(line_exceptions(source));
    }
    Ok(super::parse(path, source)?
        .map(|tree| tree_exceptions(tree.root_node(), source))
        .unwrap_or_default())
}
fn tree_exceptions(root: Node<'_>, source: &str) -> Vec<Exception> {
    let mut found = Vec::new();
    let mut pending = vec![root];
    while let Some(node) = pending.pop() {
        let comment = matches!(node.kind(), "line_comment" | "comment");
        if comment {
            let text = source.get(node.byte_range()).unwrap_or_default();
            found.extend(located(node.start_position().row + 1, text));
        }
        let mut cursor = node.walk();
        pending.extend(node.named_children(&mut cursor));
    }
    found.sort_by_key(|exception| exception.line);
    found
}
fn line_exceptions(source: &str) -> Vec<Exception> {
    source
        .lines()
        .enumerate()
        .filter_map(|(index, line)| located(index + 1, line.trim()))
        .collect()
}
fn located(line: usize, comment: &str) -> Option<Exception> {
    let marker = marker(comment)?;
    let rule = marker.rule?;
    Some(Exception {
        line,
        decision: marker.id.to_owned(),
        rule: rule.to_owned(),
    })
}
fn marker(comment: &str) -> Option<Marker<'_>> {
    let body = comment.trim_end();
    let marked = body.strip_prefix("//").or_else(|| body.strip_prefix('#'))?;
    let (label, named) = marked.strip_prefix(' ')?.split_once(": ")?;
    let (id, rule) = match named.split_once(' ') {
        Some((id, rule)) => (id, Some(rule)),
        None => (named, None),
    };
    let prefix = match (label, rule) {
        ("DECISION", _) => "D",
        ("INVARIANT", None) => "I",
        _ => return None,
    };
    let digits = id.strip_prefix(prefix)?;
    let numbered = digits.len() == 3 && digits.bytes().all(|byte| byte.is_ascii_digit());
    let single = rule.is_none_or(|rule| !rule.is_empty() && !rule.contains(char::is_whitespace));
    (numbered && single).then_some(Marker { id, rule })
}
