// DECISION: D005
use anyhow::{Result, bail};
use serde_json::Value;
use std::path::Path;

// DECISION: D015
fn outer_control(command: &str) -> bool {
    let chars: Vec<char> = command.chars().collect();
    let mut quote = None;
    let mut escaped = false;
    for (index, ch) in chars.iter().copied().enumerate() {
        let substitution = ch == '$' && chars.get(index + 1) == Some(&'(');
        let escape_start = ch == '\\' && quote != Some('\'');
        let quote_start = ch == '\'' || ch == '"';
        let shell_control = ";|&<>\n\u{60}".contains(ch) || substitution;
        if escaped {
            escaped = false;
        } else if escape_start {
            escaped = true;
        } else if let Some(q) = quote {
            let quote_end = ch == q;
            let quoted_substitution = q == '"' && (ch == '\u{60}' || substitution);
            if quote_end {
                quote = None;
            } else if quoted_substitution {
                return true;
            }
        } else if quote_start {
            quote = Some(ch);
        } else if shell_control {
            return true;
        }
    }
    quote.is_some() || escaped
}
pub fn command(event: &Value) -> Option<&str> {
    ["cmd", "command"]
        .iter()
        .find_map(|key| event["tool_input"][key].as_str())
}
pub fn arguments(command: Option<&str>) -> Result<Vec<String>> {
    let command =
        command.ok_or_else(|| anyhow::anyhow!("one top-level just invocation is required"))?;
    let has_outer_control = outer_control(command);
    if has_outer_control {
        bail!("Agent shell commands must be one top-level just invocation.");
    }
    let argv = shell_words::split(command)?;
    let not_just = argv
        .first()
        .and_then(|arg| Path::new(arg).file_name())
        .is_none_or(|name| name != "just");
    if not_just {
        bail!("Direct shell commands are disabled; use a recipe from just --list.");
    }
    Ok(argv)
}
