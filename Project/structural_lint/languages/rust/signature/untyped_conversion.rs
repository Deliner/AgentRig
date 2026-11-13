use crate::structural_lint::languages::neutral_metrics::{Code, Measurement, justified, text};
use crate::structural_lint::languages::rust::exemptions::{
    in_test_context, last_segment, type_name,
};
use crate::structural_lint::rule_identities::kind::Kind;
use crate::structural_lint::rule_identities::signature::Signature;
use tree_sitter::Node;

const CONVERSIONS: [&str; 6] = ["parse", "into", "try_into", "sum", "product", "collect"];
const TRAIT_CALLS: [(&str, &str); 5] = [
    ("From", "from"),
    ("Into", "into"),
    ("TryFrom", "try_from"),
    ("TryInto", "try_into"),
    ("FromStr", "from_str"),
];
const TRAIT_MODULES: [&str; 4] = ["std::convert", "core::convert", "std::str", "core::str"];

// DECISION: D171
pub(super) fn inspect(binding: Node<'_>, source: Code<'_>, output: &mut Vec<Measurement>) {
    let typed = binding.child_by_field_name("type").is_some();
    let plain = binding
        .child_by_field_name("pattern")
        .is_some_and(|pattern| pattern.kind() == "identifier");
    let value = binding
        .child_by_field_name("value")
        .filter(|_| plain && !typed);
    let Some(call) = value.map(unwrapped) else {
        return;
    };
    let Some(symbol) = conversion(call, source) else {
        return;
    };
    let allowed =
        justified(binding, source) || in_test_context(binding, source) || declared(call, source);
    if allowed {
        return;
    }
    output.push(Measurement {
        kind: Kind::Signature(Signature::UntypedConversion),
        line: binding.start_position().row + 1,
        symbol: symbol.into(),
        actual: 1,
    });
}
// DECISION: D181
pub(super) fn trait_path(path: Node<'_>, source: Code<'_>, output: &mut Vec<Measurement>) {
    let Some(symbol) = trait_call(path, source) else {
        return;
    };
    let allowed = bound(path, &symbol) || justified(path, source) || in_test_context(path, source);
    if allowed {
        return;
    }
    output.push(Measurement {
        kind: Kind::Signature(Signature::TraitPathCall),
        line: path.start_position().row + 1,
        symbol,
        actual: 1,
    });
}
fn trait_call(path: Node<'_>, source: Code<'_>) -> Option<String> {
    let owner = path.child_by_field_name("path")?;
    let method = text(path.child_by_field_name("name")?, source);
    let module = owner.child_by_field_name("path");
    let anchored = match owner.kind() {
        "identifier" => true,
        "scoped_identifier" => {
            module.is_some_and(|module| TRAIT_MODULES.contains(&text(module, source)))
        }
        _ => false,
    };
    let name = last_segment(owner, source);
    let paired = anchored && TRAIT_CALLS.contains(&(name, method));
    paired.then(|| [name, "::", method].concat())
}
fn bound(path: Node<'_>, symbol: &str) -> bool {
    let Some(binding) = std::iter::successors(Some(path), |node| node.parent())
        .zip(std::iter::successors(path.parent(), |node| node.parent()))
        .find(|(_, parent)| parent.kind() == "let_declaration")
        .filter(|(child, parent)| parent.child_by_field_name("value") == Some(*child))
        .map(|(_, parent)| parent)
    else {
        return false;
    };
    let typed = binding.child_by_field_name("type").is_some();
    let plain = binding
        .child_by_field_name("pattern")
        .is_some_and(|pattern| pattern.kind() == "identifier");
    let converted = binding
        .child_by_field_name("value")
        .map(unwrapped)
        .filter(|call| call.kind() == "call_expression")
        .and_then(|call| call.child_by_field_name("function"))
        == Some(path);
    let owned = plain && converted && symbol == "From::from";
    typed || owned
}
fn unwrapped(node: Node<'_>) -> Node<'_> {
    let propagated = node.kind() == "try_expression";
    match node.named_child(0) {
        Some(inner) if propagated => unwrapped(inner),
        _ => node,
    }
}
fn conversion<'a>(call: Node<'_>, source: Code<'a>) -> Option<&'a str> {
    let invoked = call.kind() == "call_expression";
    let function = call.child_by_field_name("function").filter(|_| invoked)?;
    let empty = call.child_by_field_name("arguments")?.named_child_count() == 0;
    match function.kind() {
        "field_expression" => {
            let name = text(function.child_by_field_name("field")?, source);
            let converting = empty && CONVERSIONS.contains(&name);
            converting.then_some(name)
        }
        "scoped_identifier" => {
            let owner = last_segment(function.child_by_field_name("path")?, source);
            let name = text(function.child_by_field_name("name")?, source);
            let defaulted = empty && owner == "Default" && name == "default";
            let converted = owner == "From" && name == "from";
            let generic = defaulted || converted;
            generic.then_some(name)
        }
        _ => None,
    }
}
fn declared(call: Node<'_>, source: Code<'_>) -> bool {
    let receiver = call
        .child_by_field_name("function")
        .filter(|function| function.kind() == "field_expression")
        .filter(|access| {
            access
                .child_by_field_name("field")
                .is_some_and(|field| text(field, source) == "into")
        })
        .and_then(|access| access.child_by_field_name("value"))
        .filter(|value| value.kind() == "identifier");
    let function = std::iter::successors(call.parent(), |node| node.parent())
        .find(|node| node.kind() == "function_item");
    let (Some(receiver), Some(function)) = (receiver, function) else {
        return false;
    };
    let Some(declared) = parameter_type(function, text(receiver, source), source) else {
        return false;
    };
    let generic = declared.kind() == "type_identifier";
    if generic {
        return generic_bound(function, text(declared, source), source);
    }
    opaque(declared) && into_bound(declared, source)
}
fn opaque(declared: Node<'_>) -> bool {
    match declared.kind() {
        "abstract_type" => true,
        "bounded_type" => declared.named_child(0).is_some_and(opaque),
        _ => false,
    }
}
fn parameter_type<'a>(function: Node<'a>, name: &str, source: Code<'_>) -> Option<Node<'a>> {
    let parameter = function
        .child_by_field_name("parameters")
        .map(named_children)?
        .into_iter()
        .filter(|parameter| parameter.kind() == "parameter")
        .find(|parameter| {
            parameter
                .child_by_field_name("pattern")
                .is_some_and(|pattern| {
                    pattern.kind() == "identifier" && text(pattern, source) == name
                })
        })?;
    parameter.child_by_field_name("type")
}
fn generic_bound(function: Node<'_>, name: &str, source: Code<'_>) -> bool {
    let owner = std::iter::successors(function.parent(), |node| node.parent())
        .find(|node| matches!(node.kind(), "impl_item" | "trait_item"));
    [Some(function), owner]
        .into_iter()
        .flatten()
        .flat_map(named_children)
        .filter(|part| matches!(part.kind(), "type_parameters" | "where_clause"))
        .flat_map(named_children)
        .any(|item| constrains(item, name, source))
}
fn constrains(item: Node<'_>, name: &str, source: Code<'_>) -> bool {
    let subject = item
        .child_by_field_name("name")
        .or_else(|| item.child_by_field_name("left"));
    let named = subject.is_some_and(|subject| {
        subject.kind() == "type_identifier" && text(subject, source) == name
    });
    named
        && item
            .child_by_field_name("bounds")
            .is_some_and(|bounds| into_bound(bounds, source))
}
fn into_bound(bound: Node<'_>, source: Code<'_>) -> bool {
    let named = bound.kind() == "generic_type" && type_name(bound, source) == Some("Into");
    let joined = matches!(
        bound.kind(),
        "abstract_type" | "bounded_type" | "trait_bounds"
    ) && named_children(bound)
        .into_iter()
        .any(|part| into_bound(part, source));
    named || joined
}
fn named_children(node: Node<'_>) -> Vec<Node<'_>> {
    let mut cursor = node.walk();
    node.named_children(&mut cursor).collect()
}
