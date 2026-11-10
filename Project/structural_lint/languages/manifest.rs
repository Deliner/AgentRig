use super::Analysis;
use super::measure::{Measurement, justified_line};
use crate::structural_lint::rules::kind::Kind;
use crate::structural_lint::rules::reference::Reference;
use toml::Spanned;
use toml::de::{DeString, DeTable, DeValue};

// DECISION: D098
pub(super) fn analyze(source: &str) -> Analysis {
    let mut analysis = Analysis::default();
    match DeTable::parse(source) {
        Ok(document) => scan(document.get_ref(), source, &mut analysis.measurements),
        Err(error) => {
            let offset = error.span().map_or(0, |span| span.start);
            analysis.parse_error = Some(row(source, offset) + 1);
        }
    }
    analysis
}
fn scan(root: &DeTable<'_>, source: &str, output: &mut Vec<Measurement>) {
    let workspace = root
        .get("workspace")
        .and_then(|member| member.get_ref().as_table());
    let owners = [Some(root), workspace].into_iter().flatten();
    let tools = owners
        .filter_map(|owner| {
            owner
                .get("lints")
                .and_then(|lints| lints.get_ref().as_table())
        })
        .flat_map(|lints| lints.values().filter_map(|tool| tool.get_ref().as_table()));
    for tool in tools {
        for (name, level) in tool.iter() {
            report(name, level, source, output);
        }
    }
}
fn report(
    name: &Spanned<DeString<'_>>,
    level: &Spanned<DeValue<'_>>,
    source: &str,
    output: &mut Vec<Measurement>,
) {
    let allowed = allow_level(level.get_ref());
    let other = !allowed;
    if other {
        return;
    }
    let line = row(source, name.span().start);
    let justified = justified_line(line, source);
    if justified {
        return;
    }
    output.push(Measurement {
        kind: Kind::Reference(Reference::AllowJustification),
        line: line + 1,
        symbol: name.get_ref().to_string(),
        actual: 1,
    });
}
fn allow_level(value: &DeValue<'_>) -> bool {
    let level = match value {
        DeValue::Table(table) => table.get("level").map(Spanned::get_ref),
        DeValue::String(_)
        | DeValue::Integer(_)
        | DeValue::Float(_)
        | DeValue::Boolean(_)
        | DeValue::Datetime(_)
        | DeValue::Array(_) => Some(value),
    };
    level.and_then(DeValue::as_str) == Some("allow")
}
fn row(source: &str, offset: usize) -> usize {
    source
        .bytes()
        .take(offset)
        .filter(|byte| *byte == b'\n')
        .count()
}
