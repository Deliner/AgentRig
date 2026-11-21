use super::path_syntax::{UseLeaf, leaf_name, leaf_path, module_item, scope};
use super::prelude;
use crate::structural_lint::architecture::extraction::references::{
    Reference, RustImport, Source, Target,
};
use crate::structural_lint::languages;
use tree_sitter::Node;

pub(super) fn record(source: &mut Source<'_>, node: Node<'_>, target: Target) {
    let target = match target {
        Target::RustPath { path, scope } => Target::RustPath {
            path: prelude::path(source, node, prelude::generic_path(source, node, path)),
            scope,
        },
        target @ (Target::RustMacro(_)
        | Target::RustDerive { .. }
        | Target::RustModule { .. }
        | Target::RustInclude { .. }
        | Target::PythonModule(_)
        | Target::PythonFrom { .. }) => target,
    };
    // DECISION: D146
    let test = languages::in_test_context(node, source.text);
    source.add(Reference {
        line: node.start_position().row + 1,
        target,
        test,
    });
}

pub(super) fn import_leaf(source: &mut Source<'_>, leaf: UseLeaf<'_>) {
    let Some(path) = use_leaf(source, leaf.node, &leaf.prefix) else {
        return;
    };
    let name = leaf_name(&leaf, &path);
    let node = leaf.node;
    let mut declaration = node;
    while let Some(parent) = declaration.parent() {
        let use_item = declaration.kind() == "use_declaration";
        if use_item {
            break;
        }
        declaration = parent;
    }
    let module_binding = module_item(declaration);
    if module_binding {
        source.output.rust_imports.push(RustImport {
            name,
            path,
            scope: scope(source, node),
        });
    }
}

pub(super) fn use_leaf(source: &mut Source<'_>, node: Node<'_>, prefix: &str) -> Option<String> {
    let Some(path) = leaf_path(source, node, prefix) else {
        source.unsupported(node, "Rust qualified path requires type resolution");
        return None;
    };
    record(
        source,
        node,
        Target::RustPath {
            path: path.clone(),
            scope: scope(source, node),
        },
    );
    Some(path)
}
