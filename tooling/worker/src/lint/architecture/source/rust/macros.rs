use super::super::RustMacro;
use super::{Source, Target, compact, scope};
use tree_sitter::Node;

pub(super) fn inspect(source: &mut Source<'_>, node: Node<'_>) {
    if let Some(name) = node.child_by_field_name("macro") {
        let path = compact(source.text(name));
        let source_inclusion =
            matches!(path.as_str(), "include" | "std::include" | "core::include");
        if source_inclusion {
            source.unsupported(node, "Rust source include expansion is not analyzed");
            return;
        }
        let local_import = !path.contains("::") && lexical_import(source, node, &path);
        if local_import {
            source.unsupported(node, "block-local macro imports require lexical resolution");
            return;
        }
        source.record(
            name,
            Target::RustMacro(RustMacro {
                path,
                scope: scope(source, node),
                literal: node
                    .named_children(&mut node.walk())
                    .find(|child| child.kind() == "token_tree")
                    .and_then(|tokens| literal(source, tokens)),
            }),
        );
    }
    for child in node.named_children(&mut node.walk()) {
        let tokens = child.kind() == "token_tree";
        if tokens {
            inspect_tokens(source, child);
        }
    }
}

fn lexical_import(source: &Source<'_>, node: Node<'_>, name: &str) -> bool {
    let mut parent = node.parent();
    while let Some(block) = parent {
        let lexical = block.kind() == "block";
        if lexical {
            for child in block.named_children(&mut block.walk()) {
                let matching_import =
                    child.kind() == "use_declaration" && names_import(source, child, name);
                if matching_import {
                    return true;
                }
            }
        }
        parent = block.parent();
    }
    false
}

pub(super) fn names_import(source: &Source<'_>, node: Node<'_>, name: &str) -> bool {
    let mut pending = vec![node];
    while let Some(node) = pending.pop() {
        let matches = node.kind() == "use_wildcard"
            || (node.kind() == "identifier" && source.text(node) == name);
        if matches {
            return true;
        }
        pending.extend(node.named_children(&mut node.walk()));
    }
    false
}

fn identifier(node: Node<'_>) -> bool {
    matches!(node.kind(), "identifier" | "crate" | "self" | "super")
}

fn inspect_tokens(source: &mut Source<'_>, tree: Node<'_>) {
    let children: Vec<_> = tree
        .children(&mut tree.walk())
        .filter(|node| !node.is_extra())
        .collect();
    let mut index = 0;
    while let Some(node) = children.get(index).copied() {
        let declaration = matches!(node.kind(), "use" | "mod");
        if declaration {
            source.unsupported(
                node,
                "declarations inside macro arguments require expansion and lexical analysis",
            );
        }
        let nested = node.kind() == "token_tree";
        if nested {
            inspect_tokens(source, node);
        }
        let starts_path = identifier(node);
        if starts_path {
            let (path, next) = token_path(source, &children, index);
            let macro_call = children.get(next).is_some_and(|node| node.kind() == "!");
            let arguments = macro_call.then(|| children.get(next + 1).copied().unwrap_or(node));
            record(source, node, path, arguments);
            index = next;
        } else {
            index += 1;
        }
    }
}

fn token_path(source: &Source<'_>, nodes: &[Node<'_>], start: usize) -> (String, usize) {
    let mut path = compact(source.text(nodes[start]));
    let mut index = start + 1;
    while let Some(pair) = nodes.get(index..index + 2) {
        let continuation = pair[0].kind() == "::" && identifier(pair[1]);
        let finished = !continuation;
        if finished {
            break;
        }
        path.push_str("::");
        path.push_str(&compact(source.text(pair[1])));
        index += 2;
    }
    (path, index)
}

fn record(source: &mut Source<'_>, node: Node<'_>, path: String, arguments: Option<Node<'_>>) {
    let scope = scope(source, node);
    if let Some(arguments) = arguments {
        source.record(
            node,
            Target::RustMacro(RustMacro {
                path,
                scope,
                literal: literal(source, arguments),
            }),
        );
    } else {
        let qualified = path.contains("::");
        if qualified {
            let path = super::self_path(source, node, path);
            source.record(node, Target::RustPath { path, scope });
        }
    }
}

fn literal(source: &Source<'_>, tokens: Node<'_>) -> Option<String> {
    let children: Vec<_> = tokens
        .named_children(&mut tokens.walk())
        .filter(|child| !child.is_extra())
        .collect();
    let [value] = children.as_slice() else {
        return None;
    };
    super::string_literal(source, *value)
}
