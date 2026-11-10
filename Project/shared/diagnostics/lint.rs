use crate::shared::console;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Level {
    Warning,
    Error,
}
impl Level {
    pub fn name(self) -> &'static str {
        match self {
            Self::Warning => "warning",
            Self::Error => "error",
        }
    }
    pub fn blocking(self) -> bool {
        match self {
            Self::Warning => false,
            Self::Error => true,
        }
    }
}
#[derive(Serialize)]
pub struct Diagnostic {
    pub(crate) rule: String,
    pub(crate) path: String,
    pub(crate) level: Level,
    pub(crate) actual: Option<usize>,
    pub(crate) limit: Option<usize>,
    pub(crate) skill: String,
    pub(crate) message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) line: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) symbol: Option<String>,
    pub(crate) rerun: String,
}

pub(crate) fn print_diagnostics(items: &[Diagnostic]) {
    for text in grouped(items) {
        console::stdout(text);
    }
}

fn grouped(items: &[Diagnostic]) -> Vec<String> {
    let mut groups: Vec<Vec<&Diagnostic>> = Vec::new();
    for item in items {
        let shared = groups.iter_mut().find(|group| same_cause(group[0], item));
        match shared {
            Some(group) => group.push(item),
            None => groups.push(vec![item]),
        }
    }
    groups.iter().map(|group| render(group)).collect()
}

fn same_cause(left: &Diagnostic, right: &Diagnostic) -> bool {
    let cause = |item: &Diagnostic| {
        (
            item.level,
            item.rule.clone(),
            item.limit,
            item.skill.clone(),
            item.rerun.clone(),
        )
    };
    cause(left) == cause(right)
}

// DECISION: D142
fn render(group: &[&Diagnostic]) -> String {
    let first = group[0];
    let plural = group.len() > 1;
    let suffix = if plural { "s" } else { "" };
    let location = format!("{} location{suffix}", group.len());
    let mut text = super::Guidance {
        level: first.level.name(),
        id: &first.rule,
        location: &location,
        message: "one cause, listed below",
        skill: &first.skill,
        rerun: &first.rerun,
    }
    .to_string();
    for item in group {
        text.push_str(&format!(
            "\n  {location}: {message}",
            location = format_location(item),
            message = item.message
        ));
    }
    text
}

fn format_location(item: &Diagnostic) -> String {
    match (&item.line, &item.symbol) {
        (Some(line), Some(symbol)) => format!("{}:{line} ({symbol})", item.path),
        (Some(line), None) => format!("{}:{line}", item.path),
        _ => item.path.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::{Diagnostic, Level, grouped};

    fn finding(path: &str, actual: usize, limit: usize) -> Diagnostic {
        Diagnostic {
            rule: "size".into(),
            path: path.into(),
            level: Level::Warning,
            actual: Some(actual),
            limit: Some(limit),
            skill: "skill.md".into(),
            message: format!("lines measures {actual}, exceeds {limit}"),
            line: None,
            symbol: None,
            rerun: "lint".into(),
        }
    }

    #[test]
    fn findings_with_one_cause_share_their_action_and_keep_each_location() {
        let items = [
            finding("a", 12, 10),
            finding("b", 30, 20),
            finding("c", 15, 10),
        ];
        assert_eq!(
            grouped(&items),
            [
                "WARNING [size] 2 locations: one cause, listed below. ACTION: Consider skill.md. RERUN: lint\n  a: lines measures 12, exceeds 10\n  c: lines measures 15, exceeds 10",
                "WARNING [size] 1 location: one cause, listed below. ACTION: Consider skill.md. RERUN: lint\n  b: lines measures 30, exceeds 20",
            ]
        );
    }
}
