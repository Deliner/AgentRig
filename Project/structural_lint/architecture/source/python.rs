use super::model::{Source, Target};
use tree_sitter::Node;

pub(super) fn inspect(source: &mut Source<'_>, node: Node<'_>) -> bool {
    match node.kind() {
        "import_statement" => {
            loader_aliases(source, node, "");
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
            loader_aliases(source, node, &module);
            source.record(node, Target::PythonFrom { module, names });
            return false;
        }
        "call" => dynamic(source, node),
        "identifier" | "attribute" => {
            let loader = matches!(
                source.text(node),
                "__import__" | "import_module" | "importlib.import_module"
            );
            if loader {
                source.loader_reference(node);
                return false;
            }
        }
        _ => {}
    }
    true
}

fn loader_aliases(source: &mut Source<'_>, node: Node<'_>, module: &str) {
    let loader_module =
        module == "importlib" || module.starts_with("importlib.") || module == "builtins";
    for name in node.children_by_field_name("name", &mut node.walk()) {
        let Some(original) = name.child_by_field_name("name") else {
            continue;
        };
        let imported = source.text(original);
        let loader = imported == "importlib"
            || imported.starts_with("importlib.")
            || imported == "builtins"
            || (loader_module && matches!(imported, "import_module" | "__import__"));
        if loader {
            source.unsupported(name, "aliased Python module loader needs binding analysis");
        }
    }
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
