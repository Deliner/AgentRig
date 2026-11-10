use crate::structural_lint::languages::measure::{Code, Measurement, descendants, text};
use crate::structural_lint::languages::rust::context::{
    in_test_context, justified_item, type_name,
};
use crate::structural_lint::rules::kind::Kind;
use crate::structural_lint::rules::ownership::Ownership;
use std::collections::BTreeMap;
use tree_sitter::Node;

const RAW: &[&str] = &[
    "i8", "i16", "i32", "i64", "i128", "isize", "u8", "u16", "u32", "u64", "u128", "usize", "f32",
    "f64", "bool", "char",
];

struct Definition<'a> {
    parameters: Vec<&'a str>,
    fields: usize,
}
struct File<'a> {
    definitions: BTreeMap<&'a str, Definition<'a>>,
    targets: BTreeMap<&'a str, &'a str>,
    source: Code<'a>,
}

// DECISION: D156
pub(super) fn inspect(root: Node<'_>, source: Code<'_>, output: &mut Vec<Measurement>) {
    let nodes = descendants(root);
    let implementations: Vec<_> = nodes
        .iter()
        .copied()
        .filter(|node| node.kind() == "impl_item")
        .collect();
    let file = File {
        definitions: nodes
            .iter()
            .filter(|node| node.kind() == "struct_item")
            .filter_map(|node| define(*node, source))
            .collect(),
        targets: implementations
            .iter()
            .filter(|implementation| contract(**implementation, source) == Some("Deref"))
            .filter_map(|implementation| target(*implementation, source))
            .collect(),
        source,
    };
    for implementation in implementations {
        let Some(kind) = file.classify(implementation) else {
            continue;
        };
        let allowed =
            in_test_context(implementation, source) || justified_item(implementation, source);
        if allowed {
            continue;
        }
        output.push(Measurement {
            kind: Kind::Ownership(kind),
            line: implementation.start_position().row + 1,
            symbol: owner(implementation, source).unwrap_or_default().into(),
            actual: 1,
        });
    }
}
impl File<'_> {
    fn classify(&self, implementation: Node<'_>) -> Option<Ownership> {
        let name = owner(implementation, self.source)?;
        let definition = self.definitions.get(name)?;
        match contract(implementation, self.source)? {
            "Deref" | "DerefMut" => {
                let inner = self.targets.get(name);
                let pointer = inner.is_some_and(|inner| definition.parameters.contains(inner));
                (!pointer).then_some(Ownership::WrapperDeref)
            }
            "From" => {
                let raw = definition.fields == 1 && converts_raw(implementation, self.source);
                raw.then_some(Ownership::PrimitiveFrom)
            }
            _ => None,
        }
    }
}
fn define<'a>(definition: Node<'_>, source: Code<'a>) -> Option<(&'a str, Definition<'a>)> {
    let name = text(definition.child_by_field_name("name")?, source);
    let parameters = definition
        .child_by_field_name("type_parameters")
        .map(|list| {
            let mut cursor = list.walk();
            list.named_children(&mut cursor)
                .filter_map(|parameter| parameter.child_by_field_name("name"))
                .map(|name| text(name, source))
                .collect()
        })
        .unwrap_or_default();
    let fields = definition.child_by_field_name("body").map_or(0, |body| {
        let mut cursor = body.walk();
        let ordered = body.kind() == "ordered_field_declaration_list";
        if ordered {
            return body.children_by_field_name("type", &mut cursor).count();
        }
        body.named_children(&mut cursor)
            .filter(|field| field.kind() == "field_declaration")
            .count()
    });
    Some((name, Definition { parameters, fields }))
}
fn contract<'a>(implementation: Node<'_>, source: Code<'a>) -> Option<&'a str> {
    type_name(implementation.child_by_field_name("trait")?, source)
}
fn owner<'a>(implementation: Node<'_>, source: Code<'a>) -> Option<&'a str> {
    type_name(implementation.child_by_field_name("type")?, source)
}
fn target<'a>(implementation: Node<'_>, source: Code<'a>) -> Option<(&'a str, &'a str)> {
    let body = implementation.child_by_field_name("body")?;
    let mut cursor = body.walk();
    let alias = body.named_children(&mut cursor).find(|item| {
        item.kind() == "type_item"
            && item
                .child_by_field_name("name")
                .is_some_and(|name| text(name, source) == "Target")
    })?;
    let inner = text(alias.child_by_field_name("type")?, source);
    Some((owner(implementation, source)?, inner))
}
fn converts_raw(implementation: Node<'_>, source: Code<'_>) -> bool {
    let Some(arguments) = implementation
        .child_by_field_name("trait")
        .and_then(|path| path.child_by_field_name("type_arguments"))
    else {
        return false;
    };
    let mut cursor = arguments.walk();
    let types: Vec<_> = arguments.named_children(&mut cursor).collect();
    let [argument] = types.as_slice() else {
        return false;
    };
    match argument.kind() {
        "primitive_type" => RAW.contains(&text(*argument, source)),
        "type_identifier" => text(*argument, source) == "String",
        "reference_type" => argument
            .child_by_field_name("type")
            .is_some_and(|inner| text(inner, source) == "str"),
        _ => false,
    }
}
