use super::{Source, Target};
use tree_sitter::Node;

pub(super) fn inspect(source: &mut Source<'_>, node: Node<'_>) -> bool {
    match node.kind() {
        "import_statement" | "export_statement" | "import_require_clause" => {
            if let Some(value) = node.child_by_field_name("source") {
                record(source, value);
            }
        }
        "call_expression" => call(source, node),
        _ => {}
    }
    true
}

fn record(source: &mut Source<'_>, value: Node<'_>) {
    if let Some(path) = source.literal(value) {
        source.record(value, Target::JavaScriptModule(path));
    }
}

fn call(source: &mut Source<'_>, node: Node<'_>) {
    let Some(function) = node.child_by_field_name("function") else {
        return;
    };
    let loader = matches!(
        source.text(function),
        "import" | "require" | "require.resolve"
    );
    if loader {
        let argument = node
            .child_by_field_name("arguments")
            .and_then(|args| args.named_child(0));
        match argument {
            Some(value) => record(source, value),
            None => source.unsupported(node, "module loading call has no static argument"),
        }
    }
}
