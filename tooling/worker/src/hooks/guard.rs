use anyhow::{Result, bail};
use serde_json::Value;
use std::{fs, path::Path};

// DECISION: D015
fn outer_control(command: &str) -> bool {
    let chars: Vec<char> = command.chars().collect();
    let mut quote = None;
    let mut escaped = false;
    for (index, ch) in chars.iter().copied().enumerate() {
        let substitution = ch == '$' && chars.get(index + 1) == Some(&'(');
        if escaped {
            escaped = false;
        } else if ch == '\\' && quote != Some('\'') {
            escaped = true;
        } else if let Some(q) = quote {
            if ch == q {
                quote = None;
            } else if q == '"' && (ch == '\u{60}' || substitution) {
                return true;
            }
        } else if ch == '\'' || ch == '"' {
            quote = Some(ch);
        } else if ";|&<>\n\u{60}".contains(ch) || substitution {
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
    if outer_control(command) {
        bail!("Agent shell commands must be one top-level just invocation.");
    }
    let argv = shell_words::split(command)?;
    if argv
        .first()
        .and_then(|arg| Path::new(arg).file_name())
        .is_none_or(|name| name != "just")
    {
        bail!("Direct shell commands are disabled; use a recipe from just list.");
    }
    Ok(argv)
}
pub fn validate(root: &Path, command: Option<&str>) -> Result<Vec<String>> {
    let argv = arguments(command)?;
    let catalog: Value =
        serde_json::from_slice(&fs::read(root.join("tooling/command_catalog.json"))?)?;
    if argv.len() > 1 && (argv[1].starts_with('-') || catalog.get(&argv[1]).is_none()) {
        bail!("Unknown or bypassing Just invocation; use just list.");
    }
    Ok(argv)
}
