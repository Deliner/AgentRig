use serde::Serialize;

#[derive(Serialize)]
pub struct Diagnostic {
    pub(crate) rule: String,
    pub(crate) path: String,
    pub(crate) level: String,
    pub(crate) actual: Option<u64>,
    pub(crate) limit: Option<u64>,
    pub(crate) skill: String,
    pub(crate) message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) line: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) symbol: Option<String>,
    pub(crate) rerun: String,
}

pub(crate) fn print_diagnostic(item: &Diagnostic) {
    println!(
        "{}",
        super::Guidance {
            level: &item.level,
            id: &item.rule,
            location: &format_location(item),
            message: &item.message,
            skill: &item.skill,
            rerun: &item.rerun,
        }
    );
}

fn format_location(item: &Diagnostic) -> String {
    match (&item.line, &item.symbol) {
        (Some(line), Some(symbol)) => format!("{}:{line} ({symbol})", item.path),
        (Some(line), None) => format!("{}:{line}", item.path),
        _ => item.path.clone(),
    }
}
