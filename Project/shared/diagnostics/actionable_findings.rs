use super::guidance::Guidance;
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
        let shared = groups.iter_mut().find(|group| group[0].same_cause(item));
        match shared {
            Some(group) => group.push(item),
            None => groups.push(vec![item]),
        }
    }
    groups.iter().map(|group| render(group)).collect()
}

impl Diagnostic {
    fn same_cause(&self, other: &Self) -> bool {
        let cause = |item: &Self| {
            (
                item.level,
                item.rule.clone(),
                item.limit,
                item.skill.clone(),
                item.rerun.clone(),
            )
        };
        cause(self) == cause(other)
    }
}

// DECISION: D142
fn render(group: &[&Diagnostic]) -> String {
    let first = group[0];
    let plural = group.len() > 1;
    let suffix = if plural { "s" } else { "" };
    let location = format!("{} location{suffix}", group.len());
    let mut text = Guidance {
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
