// DECISION: D037
pub fn reference(comment: &str) -> Option<&str> {
    let body = comment.trim_end();
    let marked = body.strip_prefix("//").or_else(|| body.strip_prefix('#'))?;
    let (label, id) = marked.strip_prefix(' ')?.split_once(": ")?;
    let prefix = match label {
        "DECISION" => "D",
        "INVARIANT" => "I",
        _ => return None,
    };
    let digits = id.strip_prefix(prefix)?;
    let numbered = digits.len() == 3 && digits.bytes().all(|byte| byte.is_ascii_digit());
    numbered.then_some(id)
}
