mod conditions;
mod nesting;
mod record;

use crate::structural_lint::rule_identities::kind::Kind;
use crate::structural_lint::rule_identities::reference::Reference;
use crate::structural_lint::rule_identities::signature::Signature;
use crate::structural_lint::rule_identities::structure::Structure;
pub use conditions::{boolean_expression, condition, mixed_junction};
pub use nesting::{Branch, Nest, branching, nesting};
pub use record::{Analysis, Code, Measurement, text};
use std::path::Path;
use tree_sitter::{Node, Tree};

pub fn descendants(node: Node<'_>) -> Vec<Node<'_>> {
    let mut found = Vec::new();
    let mut pending = vec![node];
    while let Some(current) = pending.pop() {
        let mut cursor = current.walk();
        pending.extend(current.named_children(&mut cursor));
        found.push(current);
    }
    found
}
pub fn reparse(path: &Path, fragment: &str) -> Option<Tree> {
    let Ok(Some(parsed)) = crate::shared::source_parsers::parse(path, fragment) else {
        return None;
    };
    let intact = !parsed.root_node().has_error();
    intact.then_some(parsed)
}
pub fn justified(node: Node<'_>, source: Code<'_>) -> bool {
    justified_line(node.start_position().row, source.as_str())
}
pub fn justified_line(row: usize, source: &str) -> bool {
    row > 0
        && source
            .lines()
            .nth(row - 1)
            .is_some_and(|line| crate::shared::source_parsers::reference(line.trim()).is_some())
}
// DECISION: D037
pub fn comment(node: Node<'_>, source: Code<'_>, output: &mut Vec<Measurement>) {
    let forbidden = crate::shared::source_parsers::reference(text(node, source)).is_none();
    if forbidden {
        output.push(Measurement {
            kind: Kind::Reference(Reference::Comment),
            line: node.start_position().row + 1,
            symbol: "comment".into(),
            actual: 1,
        });
    }
}
// DECISION: D067
pub fn credential(node: Node<'_>, source: Code<'_>, output: &mut Vec<Measurement>) {
    let found = crate::shared::source_parsers::secret(text(node, source));
    if let Some(name) = found {
        output.push(Measurement {
            kind: Kind::Reference(Reference::HardcodedSecret),
            line: node.start_position().row + 1,
            symbol: name.into(),
            actual: 1,
        });
    }
}
pub fn function(node: Node<'_>, source: Code<'_>, count: usize, output: &mut Vec<Measurement>) {
    let symbol = node
        .child_by_field_name("name")
        .map(|name| text(name, source))
        .unwrap_or("<anonymous>");
    output.push(Measurement {
        kind: Kind::Signature(Signature::ParameterCount),
        line: node.start_position().row + 1,
        symbol: symbol.into(),
        actual: count,
    });
    let has_body = node.child_by_field_name("body").is_some();
    if has_body {
        output.push(Measurement {
            kind: Kind::Structure(Structure::FunctionLines),
            line: node.start_position().row + 1,
            symbol: symbol.into(),
            actual: text(node, source)
                .lines()
                .filter(|line| !line.trim().is_empty())
                .count(),
        });
    }
}
// DECISION: D062
pub fn boolean_parameters(
    node: Node<'_>,
    source: Code<'_>,
    flag: fn(Node<'_>, Code<'_>) -> bool,
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
                kind: Kind::Signature(Signature::BooleanParameter),
                line: parameter.start_position().row + 1,
                symbol: text(parameter, source).trim().into(),
                actual: 1,
            });
        }
    }
}
