use super::model::{Loader, Source, Target};
use tree_sitter::Node;

pub(super) fn inspect(source: &mut Source<'_>, node: Node<'_>) -> bool {
    match node.kind() {
        "import_statement" | "export_statement" | "import_require_clause" => {
            if let Some(value) = node.child_by_field_name("source") {
                let loader = match node.kind() {
                    "import_require_clause" => Loader::Require,
                    _ => Loader::Import,
                };
                record(source, value, loader);
            }
        }
        "call_expression" => call(source, node),
        "identifier" | "member_expression" => {
            let loader = matches!(source.text(node), "require" | "require.resolve");
            if loader {
                source.loader_reference(node);
                return false;
            }
        }
        _ => {}
    }
    true
}

fn record(source: &mut Source<'_>, value: Node<'_>, loader: Loader) {
    if let Some(path) = source.literal(value) {
        let factory_api = matches!(path.as_str(), "module" | "node:module");
        if factory_api {
            source.unsupported(
                value,
                "Node module API can create loaders; factory bindings are not analyzed",
            );
        }
        source.record(value, Target::JavaScriptModule { path, loader });
    }
}

fn call(source: &mut Source<'_>, node: Node<'_>) {
    let Some(function) = node.child_by_field_name("function") else {
        return;
    };
    let loader = match source.text(function) {
        "import" => Some(Loader::Import),
        "require" | "require.resolve" => Some(Loader::Require),
        _ => None,
    };
    if let Some(loader) = loader {
        let argument = node
            .child_by_field_name("arguments")
            .and_then(|args| args.named_child(0));
        match argument {
            Some(value) => record(source, value, loader),
            None => source.unsupported(node, "module loading call has no static argument"),
        }
    }
}
