use crate::structural_lint::rules::control_flow::ControlFlow;
use crate::structural_lint::rules::kind::Kind;
use crate::structural_lint::rules::reference::Reference;
use crate::structural_lint::rules::signature::Signature;
use crate::structural_lint::rules::structure::Structure;
use std::collections::BTreeSet;
use tree_sitter::Node;

// DECISION: D017
pub struct Measurement {
    pub kind: Kind,
    pub line: usize,
    pub symbol: String,
    pub actual: usize,
}
#[derive(Default)]
pub struct Analysis {
    pub measurements: Vec<Measurement>,
    // DECISION: D156
    pub mixed_junctions: Vec<usize>,
    pub parse_error: Option<usize>,
}
// DECISION: D127
#[expect(clippy::string_slice)]
pub fn text<'a>(node: Node<'_>, source: &'a str) -> &'a str {
    &source[node.byte_range()]
}
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
pub fn justified(node: Node<'_>, source: &str) -> bool {
    justified_line(node.start_position().row, source)
}
pub fn justified_line(row: usize, source: &str) -> bool {
    row > 0
        && source
            .lines()
            .nth(row - 1)
            .is_some_and(|line| crate::shared::syntax::reference(line.trim()).is_some())
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
            kind: Kind::ControlFlow(ControlFlow::NamedIfCondition),
            line: node.start_position().row + 1,
            symbol: "if".into(),
            actual: 1,
        });
    }
}
// DECISION: D120
pub fn boolean_expression(node: Node<'_>, output: &mut Vec<Measurement>) {
    let nested = enclosing(node).is_some_and(connective);
    let measured = connective(node) && !nested;
    if measured {
        output.push(Measurement {
            kind: Kind::ControlFlow(ControlFlow::BooleanExpression),
            line: node.start_position().row + 1,
            symbol: "expression".into(),
            actual: operands(node),
        });
    }
}
// DECISION: D156
pub fn mixed_junction(node: Node<'_>) -> Option<usize> {
    let nested = enclosing(node).is_some_and(connective);
    let measured = connective(node) && !nested;
    let mut found = BTreeSet::new();
    junctions(node, &mut found);
    let mixed = measured && found.len() > 1;
    mixed.then_some(node.start_position().row + 1)
}
fn junctions<'a>(node: Node<'a>, found: &mut BTreeSet<&'static str>) {
    let inner = ungrouped(node);
    let operator = match inner.kind() {
        "let_chain" => Some("&&"),
        _ => inner
            .child_by_field_name("operator")
            .map(|operator| operator.kind())
            .filter(|_| connective(inner)),
    };
    let Some(operator) = operator else {
        return;
    };
    found.insert(operator);
    let mut cursor = inner.walk();
    let children: Vec<_> = inner.named_children(&mut cursor).collect();
    for child in children {
        junctions(child, found);
    }
}
fn connective(node: Node<'_>) -> bool {
    match node.kind() {
        "boolean_operator" | "let_chain" => true,
        "binary_expression" => node
            .child_by_field_name("operator")
            .is_some_and(|operator| matches!(operator.kind(), "&&" | "||")),
        _ => false,
    }
}
fn enclosing(node: Node<'_>) -> Option<Node<'_>> {
    let mut parent = node.parent()?;
    while parent.kind() == "parenthesized_expression" {
        parent = parent.parent()?;
    }
    Some(parent)
}
fn operands(node: Node<'_>) -> usize {
    let inner = ungrouped(node);
    match inner.kind() {
        "let_condition" => 0,
        _ if connective(inner) => {
            let mut cursor = inner.walk();
            inner
                .named_children(&mut cursor)
                .filter(|child| !child.is_extra())
                .map(operands)
                .sum()
        }
        _ => 1,
    }
}
fn ungrouped(node: Node<'_>) -> Node<'_> {
    let grouped = node.kind() == "parenthesized_expression";
    let mut cursor = node.walk();
    let inner = node
        .named_children(&mut cursor)
        .find(|child| !child.is_extra());
    match inner {
        Some(inner) if grouped => ungrouped(inner),
        _ => node,
    }
}
// DECISION: D037
pub fn comment(node: Node<'_>, source: &str, output: &mut Vec<Measurement>) {
    let forbidden = crate::shared::syntax::reference(text(node, source)).is_none();
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
pub fn credential(node: Node<'_>, source: &str, output: &mut Vec<Measurement>) {
    let found = crate::shared::syntax::secret(text(node, source));
    if let Some(name) = found {
        output.push(Measurement {
            kind: Kind::Reference(Reference::HardcodedSecret),
            line: node.start_position().row + 1,
            symbol: name.into(),
            actual: 1,
        });
    }
}
pub fn function(node: Node<'_>, source: &str, count: usize, output: &mut Vec<Measurement>) {
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
            kind: Kind::ControlFlow(ControlFlow::NestingDepth),
            line: node.start_position().row + 1,
            symbol: symbol.into(),
            actual: max,
        });
    }
}
fn descend(node: Node<'_>, classify: fn(Node<'_>) -> Nest, depth: usize, max: &mut usize) {
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
// DECISION: D062
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
                kind: Kind::Signature(Signature::BooleanParameter),
                line: parameter.start_position().row + 1,
                symbol: text(parameter, source).trim().into(),
                actual: 1,
            });
        }
    }
}
// DECISION: D063
pub enum Branch {
    Boundary,
    Decision,
    Plain,
}
pub fn branching(
    node: Node<'_>,
    source: &str,
    classify: fn(Node<'_>) -> Branch,
    output: &mut Vec<Measurement>,
) {
    let has_body = node.child_by_field_name("body").is_some();
    if has_body {
        let symbol = node
            .child_by_field_name("name")
            .map(|name| text(name, source))
            .unwrap_or("<anonymous>");
        let mut total = 0;
        accumulate(node, classify, &mut total);
        output.push(Measurement {
            kind: Kind::ControlFlow(ControlFlow::BranchBudget),
            line: node.start_position().row + 1,
            symbol: symbol.into(),
            actual: total,
        });
    }
}
fn accumulate(node: Node<'_>, classify: fn(Node<'_>) -> Branch, total: &mut usize) {
    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        match classify(child) {
            Branch::Boundary => {}
            Branch::Decision => {
                *total += 1;
                accumulate(child, classify, total);
            }
            Branch::Plain => accumulate(child, classify, total),
        }
    }
}
