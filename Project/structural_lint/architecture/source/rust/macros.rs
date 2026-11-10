use super::{Source, Target, compact, scope};
use crate::structural_lint::architecture::source::model::{RustItem, RustMacro};
use tree_sitter::Node;

pub(super) fn inspect(source: &mut Source<'_>, node: Node<'_>) {
    if let Some(name) = node.child_by_field_name("macro") {
        let path = compact(source.text(name));
        let source_inclusion =
            matches!(path.as_str(), "include" | "std::include" | "core::include");
        if source_inclusion {
            include(source, node);
            return;
        }
        let defines_items = item_position(node);
        if defines_items {
            source.output.rust_generated.push(scope(source, node));
        }
        let local_import = !path.contains("::") && lexical_import(source, node, &path);
        if local_import {
            source.unsupported(node, "block-local macro imports require lexical resolution");
            return;
        }
        super::record(
            source,
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

// DECISION: D136
fn item_position(node: Node<'_>) -> bool {
    let statement = node
        .parent()
        .filter(|parent| parent.kind() == "expression_statement");
    super::module_item(statement.unwrap_or(node))
}

// DECISION: D136
fn include(source: &mut Source<'_>, node: Node<'_>) {
    let tokens = node
        .named_children(&mut node.walk())
        .find(|child| child.kind() == "token_tree");
    let scope = scope(source, node);
    if let Some(file) = tokens.and_then(|tokens| literal(source, tokens)) {
        source.record(node, Target::RustInclude { file, scope });
        return;
    }
    let generated = tokens.is_some_and(|tokens| build_output(source.text(tokens)));
    if generated {
        source.output.rust_generated.push(scope);
    } else {
        source.unsupported(
            node,
            "Rust source include needs a literal path or OUT_DIR build output",
        );
    }
}

fn build_output(tokens: &str) -> bool {
    let compact: String = tokens.split_whitespace().collect();
    compact
        .strip_prefix("(concat!(env!(\"OUT_DIR\"),\"")
        .and_then(|rest| rest.strip_suffix("\"))"))
        .is_some_and(|name| !name.contains('"'))
}

// DECISION: D136
pub(super) fn definition(source: &mut Source<'_>, node: Node<'_>) {
    if let Some(name) = node.child_by_field_name("name") {
        let item = RustItem {
            name: compact(source.text(name)),
            scope: scope(source, node),
        };
        source.output.rust_macros.push(item);
    }
    for rule in node.named_children(&mut node.walk()) {
        if let Some(body) = rule.child_by_field_name("right") {
            inspect_tokens(source, body);
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
        let wildcard = node.kind() == "use_wildcard";
        let named = node.kind() == "identifier" && source.text(node) == name;
        let matches = wildcard || named;
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

// DECISION: D136
fn path_start(source: &Source<'_>, node: Node<'_>) -> bool {
    let crate_variable = node.kind() == "metavariable" && source.text(node) == "$crate";
    identifier(node) || crate_variable
}

fn inspect_tokens(source: &mut Source<'_>, tree: Node<'_>) {
    let children: Vec<_> = tree
        .children(&mut tree.walk())
        .filter(|node| !node.is_extra())
        .collect();
    let mut index = 0;
    while let Some(node) = children.get(index).copied() {
        // DECISION: D125
        super::calls::token(source, &children, index);
        let declaration = matches!(node.kind(), "use" | "mod");
        if declaration {
            source.unsupported(
                node,
                "declarations inside macro arguments require expansion and lexical analysis",
            );
        }
        let nested = matches!(node.kind(), "token_tree" | "token_repetition");
        if nested {
            inspect_tokens(source, node);
        }
        let starts_path = path_start(source, node);
        if starts_path {
            let TokenPath { path, next } = token_path(source, &children, index);
            let macro_call = children.get(next).is_some_and(|node| node.kind() == "!");
            let arguments = macro_call.then(|| children.get(next + 1).copied().unwrap_or(node));
            record(source, node, path, arguments);
            index = next;
        } else {
            index += 1;
        }
    }
}

struct TokenPath {
    path: String,
    next: usize,
}
fn token_path(source: &Source<'_>, nodes: &[Node<'_>], start: usize) -> TokenPath {
    let mut path = compact(source.text(nodes[start])).replace("$crate", "crate");
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
    TokenPath { path, next: index }
}

fn record(source: &mut Source<'_>, node: Node<'_>, path: String, arguments: Option<Node<'_>>) {
    let scope = scope(source, node);
    if let Some(arguments) = arguments {
        super::record(
            source,
            node,
            Target::RustMacro(RustMacro {
                path,
                scope,
                literal: literal(source, arguments),
            }),
        );
    } else {
        let path = super::self_path(source, node, path);
        // DECISION: D136
        let resolvable = path.contains("::") && !path.starts_with("Self::");
        if resolvable {
            super::record(source, node, Target::RustPath { path, scope });
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
