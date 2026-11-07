use crate::structural_lint::rules::kind::Kind;
use tree_sitter::Node;

// DECISION: D017
pub struct Measurement {
    pub kind: Kind,
    pub line: usize,
    pub symbol: String,
    pub actual: u64,
}
pub fn text<'a>(node: Node<'_>, source: &'a str) -> &'a str {
    &source[node.byte_range()]
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
// DECISION: D067
pub fn credential(node: Node<'_>, source: &str, output: &mut Vec<Measurement>) {
    let found = crate::shared::syntax::secret(text(node, source));
    if let Some(name) = found {
        output.push(Measurement {
            kind: Kind::HardcodedSecret,
            line: node.start_position().row + 1,
            symbol: name.into(),
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
                kind: Kind::BooleanParameter,
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
            kind: Kind::BranchBudget,
            line: node.start_position().row + 1,
            symbol: symbol.into(),
            actual: total,
        });
    }
}
fn accumulate(node: Node<'_>, classify: fn(Node<'_>) -> Branch, total: &mut u64) {
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
