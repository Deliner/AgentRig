use crate::structural_lint::languages::measure::{Code, Measurement, descendants, justified, text};
use crate::structural_lint::languages::rust::context::{in_test_context, last_segment};
use crate::structural_lint::rules::control_flow::ControlFlow;
use crate::structural_lint::rules::kind::Kind;
use tree_sitter::Node;

const CHECKS: [&str; 3] = ["exists", "is_file", "is_dir"];
const FAILURES: [&str; 6] = [
    "bail",
    "ensure",
    "panic",
    "unreachable",
    "todo",
    "unimplemented",
];
const EXITS: [&str; 3] = [
    "return_expression",
    "continue_expression",
    "break_expression",
];
const ADAPTERS: [&str; 3] = ["context", "with_context", "map_err"];
const BOUNDARIES: [&str; 2] = ["closure_expression", "function_item"];

struct Check<'tree, 'text> {
    call: Node<'tree>,
    scope: Node<'tree>,
    name: &'text str,
    path: &'text str,
}
struct Guard<'tree> {
    branch: Node<'tree>,
    absent: Option<Node<'tree>>,
    present: Option<Node<'tree>>,
}

// DECISION: D171
pub(super) fn inspect(call: Node<'_>, source: Code<'_>, output: &mut Vec<Measurement>) {
    let Some(check) = checked(call, source) else {
        return;
    };
    let Some(guard) = governing(&check, source) else {
        return;
    };
    let racing = normal(guard.absent, source) && acted(&check, &guard, source);
    let allowed = !racing || justified(call, source) || in_test_context(call, source);
    if allowed {
        return;
    }
    output.push(Measurement {
        kind: Kind::ControlFlow(ControlFlow::StaleCheck),
        line: call.start_position().row + 1,
        symbol: check.name.into(),
        actual: 1,
    });
}
fn checked<'tree, 'text>(call: Node<'tree>, source: Code<'text>) -> Option<Check<'tree, 'text>> {
    let access = call
        .child_by_field_name("function")
        .filter(|function| function.kind() == "field_expression")?;
    let name = text(access.child_by_field_name("field")?, source);
    let empty = call.child_by_field_name("arguments")?.named_child_count() == 0;
    let receiver = stripped(access.child_by_field_name("value")?, source);
    let checking = empty && CHECKS.contains(&name);
    let scope = enclosing_function(call).filter(|_| checking)?;
    Some(Check {
        call,
        scope,
        name,
        path: text(receiver, source),
    })
}
fn governing<'a>(check: &Check<'a, '_>, source: Code<'_>) -> Option<Guard<'a>> {
    let (branch, flipped) = tested(check, source)?;
    let body = branch.child_by_field_name("consequence");
    let other = branch.child_by_field_name("alternative");
    let (absent, present) = if flipped {
        (body, other)
    } else {
        (other, body)
    };
    Some(Guard {
        branch,
        absent,
        present,
    })
}
fn tested<'a>(check: &Check<'a, '_>, source: Code<'_>) -> Option<(Node<'a>, bool)> {
    let holder = container(check)?;
    let flipped = negated(check.call);
    let direct = holder.kind() == "if_expression";
    if direct {
        return Some((holder, flipped));
    }
    let name = holder
        .child_by_field_name("pattern")
        .filter(|pattern| pattern.kind() == "identifier")?;
    let (branch, mention) = first_mention(check, holder, text(name, source), source)?;
    Some((branch, flipped != negated(mention)))
}
fn container<'a>(check: &Check<'a, '_>) -> Option<Node<'a>> {
    let mut current = check.call;
    while let Some(parent) = current.parent().filter(|parent| *parent != check.scope) {
        let tested = parent.kind() == "if_expression"
            && parent.child_by_field_name("condition") == Some(current);
        let bound = parent.kind() == "let_declaration"
            && parent.child_by_field_name("value") == Some(current);
        let held = tested || bound;
        if held {
            return Some(parent);
        }
        current = parent;
    }
    None
}
fn first_mention<'a>(
    check: &Check<'a, '_>,
    binding: Node<'a>,
    name: &str,
    source: Code<'_>,
) -> Option<(Node<'a>, Node<'a>)> {
    descendants(check.scope)
        .into_iter()
        .filter(|node| node.kind() == "if_expression" && node.start_byte() >= binding.end_byte())
        .filter(|node| enclosing_function(*node) == Some(check.scope))
        .filter_map(|branch| {
            let condition = branch.child_by_field_name("condition")?;
            let mention = descendants(condition)
                .into_iter()
                .filter(|part| part.kind() == "identifier" && text(*part, source) == name)
                .min_by_key(Node::start_byte)?;
            Some((branch, mention))
        })
        .min_by_key(|(branch, _)| branch.start_byte())
}
fn negated(node: Node<'_>) -> bool {
    std::iter::successors(node.parent(), |parent| parent.parent())
        .take_while(|parent| parent.kind() == "parenthesized_expression" || negation(*parent))
        .filter(|parent| negation(*parent))
        .fold(false, |flipped, _| !flipped)
}
fn negation(node: Node<'_>) -> bool {
    node.kind() == "unary_expression"
        && node.child(0).is_some_and(|operator| operator.kind() == "!")
}
fn normal(absent: Option<Node<'_>>, source: Code<'_>) -> bool {
    absent.is_none_or(|branch| own(branch).into_iter().all(|node| !failing(node, source)))
}
fn failing(node: Node<'_>, source: Code<'_>) -> bool {
    match node.kind() {
        "try_expression" => true,
        "macro_invocation" => node
            .child_by_field_name("macro")
            .is_some_and(|name| FAILURES.contains(&last_segment(name, source))),
        "call_expression" => {
            let erroneous = node
                .child_by_field_name("function")
                .is_some_and(|function| last_segment(function, source) == "Err");
            erroneous || filesystem(node, source)
        }
        _ => false,
    }
}
fn acted(check: &Check<'_, '_>, guard: &Guard<'_>, source: Code<'_>) -> bool {
    descendants(check.scope)
        .into_iter()
        .filter(|node| {
            node.kind() == "call_expression" && node.start_byte() >= check.call.end_byte()
        })
        .filter(|node| enclosing_function(*node) == Some(check.scope))
        .filter(|node| reached(*node, guard))
        .any(|node| acts_on(node, check.path, source) && propagated(node, source))
}
fn reached(action: Node<'_>, guard: &Guard<'_>) -> bool {
    let present = guard
        .present
        .is_some_and(|branch| branch.byte_range().contains(&action.start_byte()));
    let after = action.start_byte() >= guard.branch.end_byte() && skips(guard.absent);
    present || after
}
fn skips(absent: Option<Node<'_>>) -> bool {
    absent.is_some_and(|branch| own(branch).iter().any(|node| EXITS.contains(&node.kind())))
}
fn acts_on(call: Node<'_>, path: &str, source: Code<'_>) -> bool {
    let matched = call
        .child_by_field_name("arguments")
        .is_some_and(|arguments| {
            named_children(arguments)
                .into_iter()
                .any(|argument| text(stripped(argument, source), source) == path)
        });
    matched && filesystem(call, source)
}
fn filesystem(call: Node<'_>, source: Code<'_>) -> bool {
    let function = call
        .child_by_field_name("function")
        .filter(|function| function.kind() == "scoped_identifier");
    let Some(function) = function else {
        return false;
    };
    let segments: Vec<&str> = text(function, source).split("::").map(str::trim).collect();
    let module = segments
        .iter()
        .rev()
        .skip(1)
        .any(|segment| *segment == "fs");
    let opened = matches!(segments.as_slice(), [.., "File", "open" | "create"]);
    module || opened
}
fn propagated(value: Node<'_>, source: Code<'_>) -> bool {
    let Some(parent) = value.parent() else {
        return false;
    };
    match parent.kind() {
        "try_expression" => true,
        "await_expression" => propagated(parent, source),
        "field_expression" => adapted(parent, source).is_some_and(|call| propagated(call, source)),
        _ => false,
    }
}
fn adapted<'a>(access: Node<'a>, source: Code<'_>) -> Option<Node<'a>> {
    let field = access.child_by_field_name("field")?;
    let adapter = ADAPTERS.contains(&text(field, source));
    access
        .parent()
        .filter(|call| adapter && call.kind() == "call_expression")
}
fn stripped<'a>(node: Node<'a>, source: Code<'_>) -> Node<'a> {
    let mut current = node;
    while let Some(inner) = wrapped(current, source) {
        current = inner;
    }
    current
}
fn wrapped<'a>(node: Node<'a>, source: Code<'_>) -> Option<Node<'a>> {
    match node.kind() {
        "reference_expression" => node.child_by_field_name("value"),
        "call_expression" => {
            let access = node
                .child_by_field_name("function")
                .filter(|function| function.kind() == "field_expression")?;
            let empty = node.child_by_field_name("arguments")?.named_child_count() == 0;
            let viewed = empty && text(access.child_by_field_name("field")?, source) == "as_path";
            access.child_by_field_name("value").filter(|_| viewed)
        }
        _ => None,
    }
}
fn own(node: Node<'_>) -> Vec<Node<'_>> {
    let mut found = Vec::new();
    let mut pending = vec![node];
    while let Some(current) = pending.pop() {
        let inner = named_children(current)
            .into_iter()
            .filter(|child| !BOUNDARIES.contains(&child.kind()));
        pending.extend(inner);
        found.push(current);
    }
    found
}
fn named_children(node: Node<'_>) -> Vec<Node<'_>> {
    let mut cursor = node.walk();
    node.named_children(&mut cursor).collect()
}
fn enclosing_function(node: Node<'_>) -> Option<Node<'_>> {
    std::iter::successors(node.parent(), |ancestor| ancestor.parent())
        .find(|ancestor| ancestor.kind() == "function_item")
}
