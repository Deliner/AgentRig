use super::{Source, Target};
use tree_sitter::Node;

pub(super) fn inspect(source: &mut Source<'_>, node: Node<'_>) -> bool {
    match node.kind() {
        "import_statement" => {
            for name in names(source, node) {
                source.record(node, Target::PythonModule(name));
            }
            return false;
        }
        "import_from_statement" => {
            let module = node
                .child_by_field_name("module_name")
                .map(|n| source.text(n))
                .unwrap_or("")
                .to_owned();
            let names = names(source, node);
            source.record(node, Target::PythonFrom { module, names });
            return false;
        }
        "call" => dynamic(source, node),
        _ => {}
    }
    true
}

fn names(source: &Source<'_>, node: Node<'_>) -> Vec<String> {
    let mut cursor = node.walk();
    let mut names: Vec<_> = node
        .children_by_field_name("name", &mut cursor)
        .map(|name| {
            let original = name.child_by_field_name("name").unwrap_or(name);
            source.text(original).to_owned()
        })
        .collect();
    let wildcard = node
        .named_children(&mut node.walk())
        .any(|n| n.kind() == "wildcard_import");
    if wildcard {
        names.push("*".into());
    }
    names
}

fn dynamic(source: &mut Source<'_>, node: Node<'_>) {
    let Some(function) = node.child_by_field_name("function") else {
        return;
    };
    let loader = matches!(
        source.text(function),
        "__import__" | "importlib.import_module" | "import_module"
    );
    if loader {
        let argument = node
            .child_by_field_name("arguments")
            .and_then(|args| args.named_child(0));
        match argument {
            Some(value) => {
                if let Some(module) = source.literal(value) {
                    let relative = module.starts_with('.');
                    if relative {
                        source.unsupported(
                            node,
                            "relative import_module needs runtime package resolution",
                        );
                    } else {
                        source.record(node, Target::PythonModule(module));
                    }
                }
            }
            None => source.unsupported(node, "module loading call has no static argument"),
        }
    }
}
