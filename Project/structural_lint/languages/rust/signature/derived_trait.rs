use crate::structural_lint::languages::measure::{Measurement, descendants, text};
use crate::structural_lint::languages::rust::context::{
    attributed, derives, in_test_context, last_segment, listed, type_name,
};
use crate::structural_lint::rules::kind::Kind;
use crate::structural_lint::rules::signature::Signature;
use std::collections::BTreeSet;
use tree_sitter::Node;

const SECRET_NAMES: &[&str] = &[
    "password",
    "passwd",
    "secret",
    "token",
    "api_key",
    "apikey",
    "private_key",
    "access_token",
    "refresh_token",
    "client_secret",
    "jwt",
    "cvv",
];
const CONVERSIONS: &[&str] = &["into", "to_owned", "to_string", "clone", "into_owned"];

struct Owner<'a> {
    name: &'a str,
    plain: BTreeSet<String>,
    source: &'a str,
}

// DECISION: D130
pub(super) fn secret_debug(definition: Node<'_>, source: &str, output: &mut Vec<Measurement>) {
    let exempt = in_test_context(definition, source) || !derives(definition, source, &["Debug"]);
    if exempt {
        return;
    }
    let Some(body) = definition
        .child_by_field_name("body")
        .filter(|body| body.kind() == "field_declaration_list")
    else {
        return;
    };
    let owner = definition
        .child_by_field_name("name")
        .map_or("", |name| text(name, source));
    let mut cursor = body.walk();
    for field in body.named_children(&mut cursor) {
        let (Some(name), Some(declared)) = (
            field.child_by_field_name("name"),
            field.child_by_field_name("type"),
        ) else {
            continue;
        };
        let field_name = text(name, source);
        let secret = SECRET_NAMES.contains(&field_name) && printable(declared, source);
        if secret {
            output.push(Measurement {
                kind: Kind::Signature(Signature::SecretDebug),
                line: name.start_position().row + 1,
                symbol: format!("{owner}.{field_name}"),
                actual: 1,
            });
        }
    }
}
// DECISION: D131
pub(super) fn unchecked_deserialize(root: Node<'_>, source: &str, output: &mut Vec<Measurement>) {
    let nodes = descendants(root);
    let implementations: Vec<_> = nodes
        .iter()
        .copied()
        .filter(|node| node.kind() == "impl_item")
        .collect();
    let definitions = nodes
        .into_iter()
        .filter(|node| node.kind() == "struct_item");
    for definition in definitions {
        let Some(symbol) = bypass(definition, &implementations, source) else {
            continue;
        };
        output.push(Measurement {
            kind: Kind::Signature(Signature::UncheckedDeserialize),
            line: definition.start_position().row + 1,
            symbol,
            actual: 1,
        });
    }
}
fn bypass(definition: Node<'_>, implementations: &[Node<'_>], source: &str) -> Option<String> {
    let derived = derives(definition, source, &["Deserialize"]);
    let routed = attributed(definition, source, converts) || in_test_context(definition, source);
    (derived && !routed).then_some(())?;
    let name = text(definition.child_by_field_name("name")?, source);
    let owner = Owner {
        name,
        plain: plain_fields(definition.child_by_field_name("body")?, source),
        source,
    };
    let mut constructors = Vec::new();
    let owned = implementations.iter().filter(|implementation| {
        let target = implementation.child_by_field_name("type");
        target.and_then(|path| type_name(path, source)) == Some(name)
    });
    for implementation in owned {
        let contract = implementation.child_by_field_name("trait");
        match contract.map(|path| type_name(path, source)) {
            None | Some(Some("TryFrom" | "FromStr")) => {
                constructors.extend(functions(*implementation))
            }
            Some(Some("Deserialize")) => return None,
            Some(_) => {}
        }
    }
    let checked = constructors
        .into_iter()
        .find(|function| owner.checks(*function))?;
    Some(format!(
        "{name}::{}",
        text(checked.child_by_field_name("name")?, source)
    ))
}
impl Owner<'_> {
    fn checks(&self, function: Node<'_>) -> bool {
        let (Some(parameters), Some(returned), Some(body)) = (
            function.child_by_field_name("parameters"),
            function.child_by_field_name("return_type"),
            function.child_by_field_name("body"),
        ) else {
            return false;
        };
        let constructor = self.refusable(returned) && !receives_self(parameters);
        constructor
            && parameter_names(parameters, self.source)
                .into_iter()
                .any(|parameter| {
                    checked(body, parameter, self.source) && self.stores(body, parameter)
                })
    }
    fn refusable(&self, returned: Node<'_>) -> bool {
        let wrapper = returned
            .child_by_field_name("type")
            .filter(|_| returned.kind() == "generic_type")
            .map_or("", |path| last_segment(path, self.source));
        let fallible = wrapper.ends_with("Result") || wrapper == "Option";
        let first = returned
            .child_by_field_name("type_arguments")
            .and_then(|arguments| arguments.named_child(0));
        let built = first
            .and_then(|argument| type_name(argument, self.source))
            .is_some_and(|name| self.constructs(name));
        fallible && built
    }
    fn constructs(&self, name: &str) -> bool {
        name == "Self" || name == self.name
    }
    fn stores(&self, body: Node<'_>, parameter: &str) -> bool {
        descendants(body).into_iter().any(|node| {
            let fields = match node.kind() {
                "struct_expression" => self.initialized(node, parameter),
                "call_expression" => self.positioned(node, parameter),
                _ => Vec::new(),
            };
            fields.iter().any(|field| self.plain.contains(field))
        })
    }
    fn initialized(&self, expression: Node<'_>, parameter: &str) -> Vec<String> {
        let built = expression
            .child_by_field_name("name")
            .filter(|name| name.kind() == "type_identifier")
            .is_some_and(|name| self.constructs(text(name, self.source)));
        let Some(list) = expression.child_by_field_name("body").filter(|_| built) else {
            return Vec::new();
        };
        let mut cursor = list.walk();
        let initializers: Vec<_> = list.named_children(&mut cursor).collect();
        initializers
            .into_iter()
            .filter_map(|initializer| {
                let shorthand = initializer.kind() == "shorthand_field_initializer";
                let (field, value) = if shorthand {
                    (initializer.named_child(0)?, initializer.named_child(0)?)
                } else {
                    let field = initializer.child_by_field_name("field")?;
                    (field, initializer.child_by_field_name("value")?)
                };
                passes(value, parameter, self.source).then(|| text(field, self.source).to_owned())
            })
            .collect()
    }
    fn positioned(&self, call: Node<'_>, parameter: &str) -> Vec<String> {
        let built = call
            .child_by_field_name("function")
            .filter(|function| function.kind() == "identifier")
            .is_some_and(|function| self.constructs(text(function, self.source)));
        let Some(arguments) = call.child_by_field_name("arguments").filter(|_| built) else {
            return Vec::new();
        };
        let mut cursor = arguments.walk();
        let values: Vec<_> = arguments
            .named_children(&mut cursor)
            .filter(|value| !value.is_extra())
            .collect();
        values
            .into_iter()
            .enumerate()
            .filter(|(_, value)| passes(*value, parameter, self.source))
            .map(|(position, _)| position.to_string())
            .collect()
    }
}
fn functions(implementation: Node<'_>) -> Vec<Node<'_>> {
    let Some(body) = implementation.child_by_field_name("body") else {
        return Vec::new();
    };
    let mut cursor = body.walk();
    body.named_children(&mut cursor)
        .filter(|item| item.kind() == "function_item")
        .collect()
}
fn receives_self(parameters: Node<'_>) -> bool {
    let mut cursor = parameters.walk();
    parameters.named_children(&mut cursor).any(|parameter| {
        let pattern = parameter.child_by_field_name("pattern");
        let receiver = pattern.is_some_and(|pattern| pattern.kind() == "self");
        parameter.kind() == "self_parameter" || receiver
    })
}
fn parameter_names<'a>(parameters: Node<'_>, source: &'a str) -> Vec<&'a str> {
    let mut cursor = parameters.walk();
    parameters
        .named_children(&mut cursor)
        .filter(|parameter| parameter.kind() == "parameter")
        .filter_map(|parameter| parameter.child_by_field_name("pattern"))
        .filter(|pattern| pattern.kind() == "identifier")
        .map(|pattern| text(pattern, source))
        .collect()
}
fn checked(body: Node<'_>, parameter: &str, source: &str) -> bool {
    let names = derived(body, parameter, source);
    descendants(body).into_iter().any(|node| match node.kind() {
        "if_expression" => guarded(node, &names, source),
        "macro_invocation" => ensured(node, &names, source),
        "call_expression" => gated(node, &names, source),
        _ => false,
    })
}
// DECISION: D135
fn derived<'a>(body: Node<'_>, parameter: &'a str, source: &'a str) -> Vec<&'a str> {
    let mut declarations: Vec<_> = descendants(body)
        .into_iter()
        .filter(|node| node.kind() == "let_declaration")
        .collect();
    declarations.sort_by_key(|declaration| declaration.start_byte());
    let mut names = vec![parameter];
    for declaration in declarations {
        let bound = declaration
            .child_by_field_name("pattern")
            .filter(|pattern| pattern.kind() == "identifier");
        let tainted = declaration
            .child_by_field_name("value")
            .is_some_and(|value| mentions(value, &names, source));
        names.extend(bound.filter(|_| tainted).map(|bound| text(bound, source)));
    }
    names
}
fn guarded(branch: Node<'_>, names: &[&str], source: &str) -> bool {
    let tested = branch
        .child_by_field_name("condition")
        .is_some_and(|condition| mentions(condition, names, source));
    let refused = ["consequence", "alternative"]
        .into_iter()
        .filter_map(|arm| branch.child_by_field_name(arm))
        .any(|arm| refuses(arm, source));
    tested && refused
}
fn ensured(invocation: Node<'_>, names: &[&str], source: &str) -> bool {
    let named = invocation
        .child_by_field_name("macro")
        .is_some_and(|path| last_segment(path, source) == "ensure");
    let mut cursor = invocation.walk();
    let tree = invocation
        .children(&mut cursor)
        .find(|child| child.kind() == "token_tree");
    let condition = tree.is_some_and(|tree| {
        let mut cursor = tree.walk();
        let tokens: Vec<_> = tree.children(&mut cursor).collect();
        tokens
            .into_iter()
            .take_while(|token| token.kind() != ",")
            .any(|token| mentions(token, names, source))
    });
    named && condition
}
fn gated(call: Node<'_>, names: &[&str], source: &str) -> bool {
    let Some(method) = call
        .child_by_field_name("function")
        .filter(|function| function.kind() == "field_expression")
    else {
        return false;
    };
    let named = method
        .child_by_field_name("field")
        .is_some_and(|field| matches!(text(field, source), "then" | "then_some"));
    named
        && method
            .child_by_field_name("value")
            .is_some_and(|receiver| mentions(receiver, names, source))
}
fn refuses(arm: Node<'_>, source: &str) -> bool {
    let explicit = descendants(arm).into_iter().any(|node| match node.kind() {
        "return_expression" => node
            .named_child(0)
            .is_some_and(|value| refusal(value, source)),
        "macro_invocation" => node
            .child_by_field_name("macro")
            .is_some_and(|path| matches!(last_segment(path, source), "bail" | "panic")),
        _ => false,
    });
    let block = match arm.kind() {
        "else_clause" => arm.named_child(0),
        _ => Some(arm),
    };
    let tail = block.and_then(|block| {
        let mut cursor = block.walk();
        let statements: Vec<_> = block.named_children(&mut cursor).collect();
        statements
            .into_iter()
            .rfind(|statement| !statement.is_extra())
    });
    explicit || tail.is_some_and(|value| refusal(value, source))
}
fn refusal(value: Node<'_>, source: &str) -> bool {
    match value.kind() {
        "call_expression" => value
            .child_by_field_name("function")
            .is_some_and(|function| last_segment(function, source) == "Err"),
        "identifier" => text(value, source) == "None",
        _ => false,
    }
}
fn mentions(node: Node<'_>, names: &[&str], source: &str) -> bool {
    descendants(node).into_iter().any(|token| {
        let named = token.kind() == "identifier" && names.contains(&text(token, source));
        let qualified = token
            .prev_sibling()
            .is_some_and(|previous| matches!(previous.kind(), "." | "::"));
        named && !qualified
    })
}
fn passes(value: Node<'_>, parameter: &str, source: &str) -> bool {
    match value.kind() {
        "identifier" => text(value, source) == parameter,
        "call_expression" => {
            conversion(value, source).is_some_and(|receiver| passes(receiver, parameter, source))
        }
        _ => false,
    }
}
fn conversion<'a>(call: Node<'a>, source: &str) -> Option<Node<'a>> {
    let method = call
        .child_by_field_name("function")
        .filter(|function| function.kind() == "field_expression")?;
    let name = text(method.child_by_field_name("field")?, source);
    let nullary = call.child_by_field_name("arguments")?.named_child_count() == 0;
    (CONVERSIONS.contains(&name) && nullary).then_some(())?;
    method.child_by_field_name("value")
}
fn plain_fields(body: Node<'_>, source: &str) -> BTreeSet<String> {
    let mut plain = BTreeSet::new();
    let mut routed = false;
    let mut position = 0;
    let mut cursor = body.walk();
    for child in body.named_children(&mut cursor) {
        match child.kind() {
            "attribute_item" => routed |= routes_field(child, source),
            "visibility_modifier" | "line_comment" | "block_comment" => {}
            kind => {
                let named = child
                    .child_by_field_name("name")
                    .filter(|_| kind == "field_declaration");
                let field = named.map_or_else(
                    || position.to_string(),
                    |name| text(name, source).to_owned(),
                );
                position += 1;
                let unrouted = !std::mem::take(&mut routed);
                if unrouted {
                    plain.insert(field);
                }
            }
        }
    }
    plain
}
fn converts(attribute_item: Node<'_>, source: &str) -> bool {
    listed(attribute_item, source, "serde", &["try_from", "from"])
}
fn routes_field(attribute_item: Node<'_>, source: &str) -> bool {
    listed(
        attribute_item,
        source,
        "serde",
        &["deserialize_with", "with"],
    )
}
fn printable(declared: Node<'_>, source: &str) -> bool {
    let optional = generic_argument(declared, source, "Option");
    plain(declared, source) || optional.is_some_and(|inner| plain(inner, source))
}
fn plain(declared: Node<'_>, source: &str) -> bool {
    match declared.kind() {
        "reference_type" => declared
            .child_by_field_name("type")
            .is_some_and(|target| primitive(target, source, "str")),
        "generic_type" => generic_argument(declared, source, "Vec")
            .is_some_and(|element| primitive(element, source, "u8")),
        _ => last_segment(declared, source) == "String",
    }
}
fn generic_argument<'a>(declared: Node<'a>, source: &str, base: &str) -> Option<Node<'a>> {
    let named = declared.kind() == "generic_type"
        && declared
            .child_by_field_name("type")
            .is_some_and(|path| last_segment(path, source) == base);
    named.then_some(())?;
    let arguments = declared.child_by_field_name("type_arguments")?;
    let mut cursor = arguments.walk();
    let types: Vec<_> = arguments.named_children(&mut cursor).collect();
    match types.as_slice() {
        [single] => Some(*single),
        _ => None,
    }
}
fn primitive(node: Node<'_>, source: &str, name: &str) -> bool {
    node.kind() == "primitive_type" && text(node, source) == name
}
