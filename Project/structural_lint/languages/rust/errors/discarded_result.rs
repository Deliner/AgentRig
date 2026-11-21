use crate::structural_lint::languages::neutral_metrics::{Code, Measurement, text};
use crate::structural_lint::languages::rust::exemptions::in_test_context;
use crate::structural_lint::rule_identities::{Kind, Reference};
use tree_sitter::Node;

// DECISION: D090
// DECISION: D099
// DECISION: D114
pub(super) fn inspect(node: Node<'_>, source: Code<'_>, output: &mut Vec<Measurement>) {
    let binding = node.kind() == "let_declaration";
    let symbol = if binding {
        let wildcard = node
            .child_by_field_name("pattern")
            .is_some_and(|pattern| pattern.kind() == "_");
        node.child_by_field_name("value")
            .filter(|value| wildcard && spawned(*value, source))
            .map(|_| "spawn")
    } else {
        let mut cursor = node.walk();
        let inner = node
            .named_children(&mut cursor)
            .find(|child| !child.is_extra());
        inner.and_then(|inner| {
            discarded_method(inner, source).or_else(|| spawned(inner, source).then_some("spawn"))
        })
    };
    let Some(symbol) = symbol else {
        return;
    };
    let allowed = in_test_context(node, source);
    if allowed {
        return;
    }
    output.push(Measurement {
        kind: Kind::Reference(Reference::DiscardedResult),
        line: node.start_position().row + 1,
        symbol: symbol.into(),
        actual: 1,
    });
}
fn discarded_method<'a>(inner: Node<'_>, source: Code<'a>) -> Option<&'a str> {
    let propagated = inner.kind() == "try_expression";
    let call = if propagated {
        inner.named_child(0)?
    } else {
        inner
    };
    let other = call.kind() != "call_expression";
    if other {
        return None;
    }
    let field = call
        .child_by_field_name("function")?
        .child_by_field_name("field")?;
    let unnamed = field.kind() != "field_identifier";
    if unnamed {
        return None;
    }
    let name = text(field, source);
    let dropped = if propagated {
        let nullary = call.child_by_field_name("arguments")?.named_child_count() == 0;
        ["status", "output"].contains(&name) && nullary
    } else {
        ["ok", "err", "map", "and_then", "map_or", "map_or_else"].contains(&name)
    };
    dropped.then_some(name)
}
// DECISION: D155
fn spawned(value: Node<'_>, source: Code<'_>) -> bool {
    let mut current = value;
    loop {
        let propagated = current.kind() == "try_expression";
        if propagated {
            let Some(inner) = current.named_child(0) else {
                return false;
            };
            current = inner;
            continue;
        }
        let Some(function) = current
            .child_by_field_name("function")
            .filter(|_| current.kind() == "call_expression")
        else {
            return false;
        };
        let name = match function.kind() {
            "identifier" => text(function, source),
            "scoped_identifier" | "field_expression" => function
                .child_by_field_name("name")
                .or_else(|| function.child_by_field_name("field"))
                .map_or("", |name| text(name, source)),
            _ => "",
        };
        let unwrapped =
            matches!(name, "unwrap" | "expect") && function.kind() == "field_expression";
        let Some(receiver) = unwrapped
            .then(|| function.child_by_field_name("value"))
            .flatten()
        else {
            return name == "spawn";
        };
        current = receiver;
    }
}
