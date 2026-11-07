use super::context::{in_test_context, plain_name, type_name};
use crate::structural_lint::languages::measure::{Measurement, descendants, justified, text};
use crate::structural_lint::rules::kind::Kind;
use crate::structural_lint::rules::signature::Signature;
use std::collections::BTreeSet;
use tree_sitter::Node;

type Fields = BTreeSet<String>;

struct Implementation<'a> {
    node: Node<'a>,
    owner: &'a str,
    contract: Option<&'a str>,
}
struct Owner<'s, 'a> {
    name: &'a str,
    root: Node<'a>,
    implementations: &'s [Implementation<'a>],
    source: &'a str,
}

// DECISION: D119
pub(super) fn inspect<'a>(root: Node<'a>, source: &'a str, output: &mut Vec<Measurement>) {
    let implementations: Vec<_> = descendants(root)
        .into_iter()
        .filter_map(|node| implementation(node, source))
        .collect();
    for ordering in &implementations {
        let owner = Owner {
            name: ordering.owner,
            root,
            implementations: &implementations,
            source,
        };
        let allowed = justified(ordering.node, source) || in_test_context(ordering.node, source);
        let symbol = if allowed {
            None
        } else {
            mismatch(ordering, &owner)
        };
        if let Some(symbol) = symbol {
            output.push(Measurement {
                kind: Kind::Signature(Signature::OrdEqMismatch),
                line: ordering.node.start_position().row + 1,
                symbol,
                actual: 1,
            });
        }
    }
}
fn implementation<'a>(node: Node<'a>, source: &'a str) -> Option<Implementation<'a>> {
    (node.kind() == "impl_item").then_some(())?;
    Some(Implementation {
        node,
        owner: type_name(node.child_by_field_name("type")?, source)?,
        contract: node
            .child_by_field_name("trait")
            .and_then(|contract| plain_name(contract, source)),
    })
}
fn mismatch<'a>(ordering: &Implementation<'a>, owner: &Owner<'_, 'a>) -> Option<String> {
    let method = match ordering.contract? {
        "Ord" => "cmp",
        "PartialOrd" if !owner.implements("Ord") => "partial_cmp",
        _ => return None,
    };
    let order = owner.reads(method_body(ordering.node, method, owner.source)?, method)?;
    let equality = owner.equality()?;
    let sides = [
        ("equality-only", equality.difference(&order)),
        ("order-only", order.difference(&equality)),
    ];
    let parts: Vec<String> = sides
        .into_iter()
        .filter_map(|(label, fields)| {
            let fields: Vec<&str> = fields.map(String::as_str).collect();
            (!fields.is_empty()).then(|| format!("{label} {}", fields.join(", ")))
        })
        .collect();
    (!parts.is_empty()).then(|| format!("{}: {}", owner.name, parts.join("; ")))
}
impl<'a> Owner<'_, 'a> {
    fn own(&self) -> impl Iterator<Item = &Implementation<'a>> {
        self.implementations
            .iter()
            .filter(|implementation| implementation.owner == self.name)
    }
    fn implements(&self, contract: &str) -> bool {
        self.own()
            .any(|implementation| implementation.contract == Some(contract))
    }
    fn equality(&self) -> Option<Fields> {
        let manual = self
            .own()
            .find(|implementation| implementation.contract == Some("PartialEq"));
        match manual {
            Some(manual) => self.reads(method_body(manual.node, "eq", self.source)?, "eq"),
            None => derived_fields(self.root, self.name, self.source),
        }
    }
    fn reads(&self, body: Node<'a>, method: &'a str) -> Option<Fields> {
        self.collect(body, &mut BTreeSet::from([method]))
    }
    fn collect(&self, body: Node<'a>, visited: &mut BTreeSet<&'a str>) -> Option<Fields> {
        let mut fields = Fields::new();
        let receivers = descendants(body)
            .into_iter()
            .filter(|node| receiver(*node, self.source));
        for receiver in receivers {
            let access = field_access(receiver)?;
            let name = text(access.child_by_field_name("field")?, self.source);
            let method = called(access);
            let reached = if method {
                self.call(name, visited)?
            } else {
                Fields::from([name.to_owned()])
            };
            fields.extend(reached);
        }
        Some(fields)
    }
    fn call(&self, name: &'a str, visited: &mut BTreeSet<&'a str>) -> Option<Fields> {
        let repeated = !visited.insert(name);
        if repeated {
            return Some(Fields::new());
        }
        let bodies: Vec<_> = self
            .own()
            .filter_map(|implementation| method_body(implementation.node, name, self.source))
            .collect();
        (!bodies.is_empty()).then_some(())?;
        let mut fields = Fields::new();
        for body in bodies {
            fields.extend(self.collect(body, visited)?);
        }
        Some(fields)
    }
}
fn method_body<'a>(implementation: Node<'a>, name: &str, source: &str) -> Option<Node<'a>> {
    let body = implementation.child_by_field_name("body")?;
    let mut cursor = body.walk();
    let method = body.named_children(&mut cursor).find(|item| {
        let named = item
            .child_by_field_name("name")
            .is_some_and(|identifier| text(identifier, source) == name);
        item.kind() == "function_item" && named
    })?;
    method.child_by_field_name("body")
}
fn receiver(node: Node<'_>, source: &str) -> bool {
    node.kind() == "self" || (node.kind() == "identifier" && text(node, source) == "self")
}
fn field_access(receiver: Node<'_>) -> Option<Node<'_>> {
    receiver.parent().filter(|parent| {
        parent.kind() == "field_expression" && parent.child_by_field_name("value") == Some(receiver)
    })
}
fn called(access: Node<'_>) -> bool {
    access.parent().is_some_and(|parent| {
        parent.kind() == "call_expression" && parent.child_by_field_name("function") == Some(access)
    })
}
fn derived_fields(root: Node<'_>, name: &str, source: &str) -> Option<Fields> {
    let definition = descendants(root).into_iter().find(|node| {
        let named = node
            .child_by_field_name("name")
            .is_some_and(|identifier| text(identifier, source) == name);
        node.kind() == "struct_item" && named
    })?;
    derives_equality(definition, source).then_some(())?;
    let body = definition
        .child_by_field_name("body")
        .filter(|body| body.kind() == "field_declaration_list")?;
    let mut cursor = body.walk();
    let fields = body
        .named_children(&mut cursor)
        .filter_map(|field| field.child_by_field_name("name"))
        .map(|field| text(field, source).to_owned())
        .collect();
    Some(fields)
}
fn derives_equality(definition: Node<'_>, source: &str) -> bool {
    let mut sibling = definition.prev_named_sibling();
    while let Some(previous) = sibling.filter(|node| {
        matches!(
            node.kind(),
            "attribute_item" | "line_comment" | "block_comment"
        )
    }) {
        let derived = previous
            .named_child(0)
            .is_some_and(|attribute| derives_partial_eq(attribute, source));
        if derived {
            return true;
        }
        sibling = previous.prev_named_sibling();
    }
    false
}
fn derives_partial_eq(attribute: Node<'_>, source: &str) -> bool {
    let derive = attribute.kind() == "attribute"
        && attribute
            .named_child(0)
            .is_some_and(|path| text(path, source) == "derive");
    let arguments = attribute.child_by_field_name("arguments");
    derive
        && arguments.is_some_and(|arguments| {
            descendants(arguments)
                .into_iter()
                .any(|token| token.kind() == "identifier" && text(token, source) == "PartialEq")
        })
}
