use super::context::justified_item;
use crate::structural_lint::languages::measure::{Code, Measurement, text};
use crate::structural_lint::rules::kind::Kind;
use crate::structural_lint::rules::reference::Reference;
use tree_sitter::Node;

// DECISION: D065
// DECISION: D103
pub(super) fn inspect(node: Node<'_>, source: Code<'_>, output: &mut Vec<Measurement>) {
    if let Some(symbol) = site(node) {
        report(node, symbol, source, output);
    }
    let tree = matches!(node.kind(), "token_tree" | "token_repetition");
    let plain = !tree;
    if plain {
        return;
    }
    let mut cursor = node.walk();
    for token in node
        .children(&mut cursor)
        .filter(|child| child.kind() == "unsafe" && !attribute_opener(*child, source))
    {
        report(token, "unsafe", source, output);
    }
}
fn attribute_opener(token: Node<'_>, source: Code<'_>) -> bool {
    let previous = token.prev_sibling().map(|node| node.kind());
    let head = token.parent().and_then(|tree| tree.prev_sibling());
    let hashed = head.is_some_and(|hash| hash.kind() == "#");
    let direct = hashed && previous == Some("[");
    let conditional = head.is_some_and(|name| text(name, source) == "cfg_attr");
    let argument = conditional && previous == Some(",");
    let arguments = token
        .next_sibling()
        .and_then(|next| next.child(0))
        .is_some_and(|open| open.kind() == "(");
    let positioned = direct || argument;
    positioned && arguments
}
fn site(node: Node<'_>) -> Option<&'static str> {
    match node.kind() {
        "unsafe_block" => Some("unsafe"),
        "impl_item" | "trait_item" => keyword(node, "unsafe").then_some("unsafe"),
        "function_item" | "function_signature_item" => modifiers(node)
            .is_some_and(|modifiers| keyword(modifiers, "unsafe"))
            .then_some("unsafe"),
        "static_item" => keyword(node, "mutable_specifier").then_some("static mut"),
        _ => None,
    }
}
fn modifiers(node: Node<'_>) -> Option<Node<'_>> {
    let mut cursor = node.walk();
    node.children(&mut cursor)
        .find(|child| child.kind() == "function_modifiers")
}
fn keyword(node: Node<'_>, kind: &str) -> bool {
    let mut cursor = node.walk();
    node.children(&mut cursor).any(|child| child.kind() == kind)
}
fn report(node: Node<'_>, symbol: &str, source: Code<'_>, output: &mut Vec<Measurement>) {
    let allowed = justified_item(node, source);
    if allowed {
        return;
    }
    output.push(Measurement {
        kind: Kind::Reference(Reference::UnsafeJustification),
        line: node.start_position().row + 1,
        symbol: symbol.into(),
        actual: 1,
    });
}
