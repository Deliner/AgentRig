use crate::structural_lint::languages::measure::{Measurement, justified, text};
use crate::structural_lint::rules::kind::Kind;
use crate::structural_lint::rules::reference::Reference;
use tree_sitter::Node;

// DECISION: D074
// DECISION: D080
// DECISION: D098
const SUPPRESSIONS: &[&str] = &["allow", "expect", "ignore"];

pub(super) fn attribute(node: Node<'_>, source: &str, output: &mut Vec<Measurement>) {
    let Some(name) = suppression(node, source) else {
        return;
    };
    let kind = match name {
        "ignore" => Kind::Reference(Reference::DisabledTest),
        _ => Kind::Reference(Reference::AllowJustification),
    };
    let unjustified = !justified(node, source);
    if unjustified {
        output.push(Measurement {
            kind,
            line: node.start_position().row + 1,
            symbol: name.into(),
            actual: 1,
        });
    }
}
fn suppression<'a>(node: Node<'_>, source: &'a str) -> Option<&'a str> {
    let attribute = node
        .named_children(&mut node.walk())
        .find(|child| child.kind() == "attribute")?;
    let name = text(attribute.named_child(0)?, source);
    let conditional = name == "cfg_attr";
    if conditional {
        return conditional_head(attribute.child_by_field_name("arguments")?, source);
    }
    SUPPRESSIONS.contains(&name).then_some(name)
}
fn conditional_head<'a>(arguments: Node<'_>, source: &'a str) -> Option<&'a str> {
    let mut cursor = arguments.walk();
    let mut argument = false;
    for token in arguments.children(&mut cursor) {
        let separator = token.kind() == ",";
        if separator {
            argument = true;
            continue;
        }
        let name = text(token, source);
        let head = argument && token.kind() == "identifier" && SUPPRESSIONS.contains(&name);
        if head {
            return Some(name);
        }
        argument = false;
    }
    None
}
