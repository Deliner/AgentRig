mod python;
mod registry;
mod rust;
#[cfg(test)]
mod tests;
use super::rules::kind::Kind;
pub use registry::{HANDLERS, handler};

use anyhow::{Context, Result};
use std::path::Path;
use tree_sitter::Node;

// DECISION: D017
pub struct Measurement {
    pub kind: Kind,
    pub line: usize,
    pub symbol: String,
    pub actual: u64,
}
pub struct Analysis {
    pub measurements: Vec<Measurement>,
    pub parse_error: Option<usize>,
}
pub fn analyze(path: &Path, source: &str) -> Result<Analysis> {
    let handler = handler(path).context("unsupported source language")?;
    let inspect_node = handler
        .inspect
        .context("language has no scalar measurement handler")?;
    let tree =
        crate::shared::syntax::parse(path, source)?.context("unsupported source language")?;
    let mut analysis = Analysis {
        measurements: Vec::new(),
        parse_error: None,
    };
    let valid = !tree.root_node().has_error();
    let mut pending = vec![tree.root_node()];
    while let Some(node) = pending.pop() {
        let invalid = node.is_error() || node.is_missing();
        if invalid {
            analysis
                .parse_error
                .get_or_insert(node.start_position().row + 1);
        }
        let inspect = valid && node.is_named();
        if inspect {
            inspect_node(node, source, &mut analysis.measurements);
        }
        let mut cursor = node.walk();
        pending.extend(
            node.children(&mut cursor)
                .collect::<Vec<_>>()
                .into_iter()
                .rev(),
        );
    }
    Ok(analysis)
}
pub fn text<'a>(node: Node<'_>, source: &'a str) -> &'a str {
    &source[node.byte_range()]
}
pub fn named_value(node: Node<'_>) -> bool {
    match node.kind() {
        "identifier" | "self" => true,
        "parenthesized_expression" => {
            let mut cursor = node.walk();
            node.named_children(&mut cursor)
                .find(|child| !child.is_extra())
                .is_some_and(named_value)
        }
        "attribute" => node.child_by_field_name("object").is_some_and(named_value),
        "field_expression" => {
            let named_field = node
                .child_by_field_name("field")
                .is_some_and(|field| field.kind() == "field_identifier");
            named_field && node.child_by_field_name("value").is_some_and(named_value)
        }
        "scoped_identifier" => node.child_by_field_name("path").is_some_and(named_value),
        "crate" | "super" => true,
        _ => false,
    }
}
pub fn condition(node: Node<'_>, condition: Node<'_>, output: &mut Vec<Measurement>) {
    let invalid = !named_value(condition);
    if invalid {
        output.push(Measurement {
            kind: Kind::NamedIfCondition,
            line: node.start_position().row + 1,
            symbol: "if".into(),
            actual: 1,
        });
    }
}
// DECISION: D037
pub fn comment(node: Node<'_>, source: &str, output: &mut Vec<Measurement>) {
    let forbidden = crate::shared::syntax::reference(text(node, source)).is_none();
    if forbidden {
        output.push(Measurement {
            kind: Kind::CommentReference,
            line: node.start_position().row + 1,
            symbol: "comment".into(),
            actual: 1,
        });
    }
}
pub fn function(node: Node<'_>, source: &str, count: u64, output: &mut Vec<Measurement>) {
    let symbol = node
        .child_by_field_name("name")
        .map(|name| text(name, source))
        .unwrap_or("<anonymous>");
    output.push(Measurement {
        kind: Kind::ParameterCount,
        line: node.start_position().row + 1,
        symbol: symbol.into(),
        actual: count,
    });
    let has_body = node.child_by_field_name("body").is_some();
    if has_body {
        output.push(Measurement {
            kind: Kind::FunctionLines,
            line: node.start_position().row + 1,
            symbol: symbol.into(),
            actual: text(node, source)
                .lines()
                .filter(|line| !line.trim().is_empty())
                .count() as u64,
        });
    }
}
// DECISION: D061
pub enum Nest {
    Increment,
    Boundary,
    Plain,
}
pub fn nesting(
    node: Node<'_>,
    source: &str,
    classify: fn(Node<'_>) -> Nest,
    output: &mut Vec<Measurement>,
) {
    let has_body = node.child_by_field_name("body").is_some();
    if has_body {
        let symbol = node
            .child_by_field_name("name")
            .map(|name| text(name, source))
            .unwrap_or("<anonymous>");
        let mut max = 0;
        descend(node, classify, 0, &mut max);
        output.push(Measurement {
            kind: Kind::NestingDepth,
            line: node.start_position().row + 1,
            symbol: symbol.into(),
            actual: max,
        });
    }
}
fn descend(node: Node<'_>, classify: fn(Node<'_>) -> Nest, depth: u64, max: &mut u64) {
    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        match classify(child) {
            Nest::Boundary => {}
            Nest::Increment => {
                let inner = depth + 1;
                *max = (*max).max(inner);
                descend(child, classify, inner, max);
            }
            Nest::Plain => descend(child, classify, depth, max),
        }
    }
}
pub fn boolean_parameters(
    node: Node<'_>,
    source: &str,
    flag: fn(Node<'_>, &str) -> bool,
    output: &mut Vec<Measurement>,
) {
    let Some(parameters) = node.child_by_field_name("parameters") else {
        return;
    };
    let mut cursor = parameters.walk();
    for parameter in parameters.named_children(&mut cursor) {
        let boolean = flag(parameter, source);
        if boolean {
            output.push(Measurement {
                kind: Kind::BooleanParameter,
                line: parameter.start_position().row + 1,
                symbol: text(parameter, source).trim().into(),
                actual: 1,
            });
        }
    }
}
