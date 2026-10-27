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

pub(crate) fn print_diagnostics(items: &[Diagnostic]) {
    for text in grouped(items) {
        println!("{text}");
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
            item.level.clone(),
            item.rule.clone(),
            item.limit,
            item.skill.clone(),
            item.rerun.clone(),
        )
    };
    cause(left) == cause(right)
}

fn render(group: &[&Diagnostic]) -> String {
    let first = group[0];
    let single = group.len() == 1;
    let (location, message) = if single {
        (format_location(first), first.message.clone())
    } else {
        (
            format!("{} locations", group.len()),
            "one cause, findings below".to_owned(),
        )
    };
    let mut text = super::Guidance {
        level: &first.level,
        id: &first.rule,
        location: &location,
        message: &message,
        skill: &first.skill,
        rerun: &first.rerun,
    }
    .to_string();
    for item in group.iter().filter(|_| !single) {
        text.push_str(&format!("\n  {}: {}", format_location(item), item.message));
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
    use super::{Diagnostic, grouped};

    fn finding(path: &str, actual: u64, limit: u64) -> Diagnostic {
        Diagnostic {
            rule: "size".into(),
            path: path.into(),
            level: "warning".into(),
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
                "WARNING [size] 2 locations: one cause, findings below. ACTION: Consider skill.md. RERUN: lint\n  a: lines measures 12, exceeds 10\n  c: lines measures 15, exceeds 10",
                "WARNING [size] b: lines measures 30, exceeds 20. ACTION: Consider skill.md. RERUN: lint",
            ]
        );
    }
}
