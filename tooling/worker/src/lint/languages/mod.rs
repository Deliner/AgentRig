mod python;
mod rust;

use anyhow::{Context, Result};
use std::path::Path;
use tree_sitter::{Node, Parser};

// DECISION: D017
pub struct Measurement {
    pub kind: &'static str,
    pub line: usize,
    pub symbol: String,
    pub actual: u64,
}
pub struct Analysis {
    pub measurements: Vec<Measurement>,
    pub parse_error: Option<usize>,
}
pub fn supports(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|ext| ext.to_str()),
        Some("rs" | "py" | "pyi")
    )
}
pub fn analyze(path: &Path, source: &str) -> Result<Analysis> {
    let is_rust = path.extension().is_some_and(|ext| ext == "rs");
    let language = if is_rust {
        tree_sitter_rust::LANGUAGE
    } else {
        tree_sitter_python::LANGUAGE
    };
    let mut parser = Parser::new();
    parser.set_language(&language.into())?;
    let tree = parser
        .parse(source, None)
        .context("parser returned no tree")?;
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
            if is_rust {
                rust::inspect(node, source, &mut analysis.measurements);
            } else {
                python::inspect(node, source, &mut analysis.measurements);
            }
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
            kind: "named-if-condition",
            line: node.start_position().row + 1,
            symbol: "if".into(),
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
        kind: "parameter-count",
        line: node.start_position().row + 1,
        symbol: symbol.into(),
        actual: count,
    });
    let has_body = node.child_by_field_name("body").is_some();
    if has_body {
        output.push(Measurement {
            kind: "function-lines",
            line: node.start_position().row + 1,
            symbol: symbol.into(),
            actual: text(node, source)
                .lines()
                .filter(|line| !line.trim().is_empty())
                .count() as u64,
        });
    }
}
