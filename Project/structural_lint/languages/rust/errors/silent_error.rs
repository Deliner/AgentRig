use crate::structural_lint::languages::measure::{Code, Measurement, descendants, justified, text};
use crate::structural_lint::languages::rust::context::{in_test_context, last_segment};
use crate::structural_lint::rule_identities::kind::Kind;
use crate::structural_lint::rule_identities::reference::Reference;
use tree_sitter::Node;

const ERROR_ARMS: &[&str] = &["Err(_)", "Err(..)"];

// DECISION: D097
// DECISION: D165
pub(super) fn collapsed_chain(node: Node<'_>, source: Code<'_>, output: &mut Vec<Measurement>) {
    let Some(function) = node.child_by_field_name("function") else {
        return;
    };
    let Some(field) = function
        .child_by_field_name("field")
        .filter(|field| field.kind() == "field_identifier")
    else {
        return;
    };
    let Some(receiver) = function.child_by_field_name("value") else {
        return;
    };
    let Some(arguments) = node.child_by_field_name("arguments") else {
        return;
    };
    let name = text(field, source);
    let dropped = match (name, values(arguments).as_slice()) {
        ("flatten", []) => read_dir_rooted(receiver, source),
        ("map_while", [argument]) => {
            result_ok(*argument, source) && read_dir_rooted(receiver, source)
        }
        ("filter_map" | "flat_map", [argument]) => {
            let clippy = lines_call(receiver, source) && plain_ok(*argument, source);
            result_ok(*argument, source) && !clippy
        }
        _ => false,
    };
    let allowed = !dropped || justified(node, source) || in_test_context(node, source);
    if allowed {
        return;
    }
    output.push(Measurement {
        kind: Kind::Reference(Reference::SilentError),
        line: node.start_position().row + 1,
        symbol: name.into(),
        actual: 1,
    });
}
fn result_ok(argument: Node<'_>, source: Code<'_>) -> bool {
    ok_path(argument, source) || ok_receiver(argument, source).is_some()
}
fn ok_path(argument: Node<'_>, source: Code<'_>) -> bool {
    argument.kind() == "scoped_identifier" && text(argument, source) == "Result::ok"
}
fn ok_receiver<'a>(argument: Node<'a>, source: Code<'_>) -> Option<Node<'a>> {
    let closure = argument.kind() == "closure_expression";
    let parameters = argument
        .child_by_field_name("parameters")
        .filter(|_| closure)?;
    let body = argument
        .child_by_field_name("body")
        .filter(|_| parameters.named_child_count() == 1)?;
    let empty = body
        .child_by_field_name("arguments")
        .is_some_and(|arguments| arguments.named_child_count() == 0);
    let function = body
        .child_by_field_name("function")
        .filter(|_| body.kind() == "call_expression" && empty)?;
    let ok = function
        .child_by_field_name("field")
        .is_some_and(|field| text(field, source) == "ok");
    function.child_by_field_name("value").filter(|_| ok)
}
fn values(arguments: Node<'_>) -> Vec<Node<'_>> {
    let mut cursor = arguments.walk();
    arguments
        .named_children(&mut cursor)
        .filter(|child| !child.is_extra())
        .collect()
}
fn lines_call(receiver: Node<'_>, source: Code<'_>) -> bool {
    receiver.kind() == "call_expression"
        && receiver
            .child_by_field_name("function")
            .is_some_and(|function| {
                function.kind() == "field_expression"
                    && callee_name(function, source) == Some("lines")
            })
}
fn plain_ok(argument: Node<'_>, source: Code<'_>) -> bool {
    let parameter = argument
        .child_by_field_name("parameters")
        .and_then(|parameters| parameters.named_child(0));
    let itself =
        ok_receiver(argument, source)
            .zip(parameter)
            .is_some_and(|(receiver, parameter)| {
                receiver.kind() == "identifier" && text(receiver, source) == text(parameter, source)
            });
    ok_path(argument, source) || itself
}
fn read_dir_rooted(receiver: Node<'_>, source: Code<'_>) -> bool {
    let mut current = receiver;
    while let Some(inner) = unwrapped(current, source) {
        current = inner;
    }
    let call = current.kind() == "call_expression";
    call && current
        .child_by_field_name("function")
        .is_some_and(|function| callee_name(function, source) == Some("read_dir"))
}
fn unwrapped<'a>(node: Node<'a>, source: Code<'_>) -> Option<Node<'a>> {
    let propagated = node.kind() == "try_expression";
    if propagated {
        return node.named_child(0);
    }
    let call = node.kind() == "call_expression";
    let other = !call;
    if other {
        return None;
    }
    let function = node.child_by_field_name("function")?;
    let adapter =
        function.kind() == "field_expression" && callee_name(function, source) == Some("into_iter");
    if adapter {
        return function.child_by_field_name("value");
    }
    None
}
fn callee_name<'a>(function: Node<'_>, source: Code<'a>) -> Option<&'a str> {
    match function.kind() {
        "identifier" => Some(text(function, source)),
        "scoped_identifier" => function
            .child_by_field_name("name")
            .map(|name| text(name, source)),
        "field_expression" => function
            .child_by_field_name("field")
            .map(|field| text(field, source)),
        _ => None,
    }
}
pub(super) fn verdict_arm(node: Node<'_>, source: Code<'_>, output: &mut Vec<Measurement>) {
    let swallowed = node
        .child_by_field_name("pattern")
        .is_some_and(|pattern| error_pattern(pattern, source))
        && node
            .child_by_field_name("value")
            .is_some_and(|value| answers(value, source));
    if swallowed {
        report_verdict(node, source, output);
    }
}
pub(super) fn verdict_if(node: Node<'_>, source: Code<'_>, output: &mut Vec<Measurement>) {
    let swallowed = node
        .child_by_field_name("condition")
        .and_then(|condition| condition.child_by_field_name("pattern"))
        .is_some_and(|pattern| error_pattern(pattern, source))
        && node
            .child_by_field_name("consequence")
            .is_some_and(|block| answers(block, source));
    if swallowed {
        report_verdict(node, source, output);
    }
}
fn report_verdict(node: Node<'_>, source: Code<'_>, output: &mut Vec<Measurement>) {
    let Some(function) = verdict_function(node, source) else {
        return;
    };
    let allowed = justified(node, source) || in_test_context(node, source);
    if allowed {
        return;
    }
    output.push(Measurement {
        kind: Kind::Reference(Reference::SilentError),
        line: node.start_position().row + 1,
        symbol: function.into(),
        actual: 1,
    });
}
fn error_pattern(pattern: Node<'_>, source: Code<'_>) -> bool {
    let compact: String = text(pattern, source).split_whitespace().collect();
    ERROR_ARMS.contains(&compact.as_str())
}
fn answers(node: Node<'_>, source: Code<'_>) -> bool {
    match node.kind() {
        "block" | "expression_statement" | "return_expression" => {
            let mut cursor = node.walk();
            let mut inner = node
                .named_children(&mut cursor)
                .filter(|child| !child.is_extra());
            let first = inner.next();
            let lone = inner.next().is_none();
            lone && first.is_some_and(|child| answers(child, source))
        }
        "call_expression" => verdict_call(node, source),
        _ => false,
    }
}
fn verdict_call(node: Node<'_>, source: Code<'_>) -> bool {
    let constructor = node
        .child_by_field_name("function")
        .is_some_and(|function| function.kind() == "identifier" && text(function, source) == "Ok");
    let flag = node
        .child_by_field_name("arguments")
        .is_some_and(|arguments| {
            let mut cursor = arguments.walk();
            let mut values = arguments
                .named_children(&mut cursor)
                .filter(|child| !child.is_extra());
            let first = values.next();
            values.next().is_none() && first.is_some_and(|value| value.kind() == "boolean_literal")
        });
    constructor && flag
}
fn verdict_function<'a>(node: Node<'_>, source: Code<'a>) -> Option<&'a str> {
    let mut current = node.parent()?;
    loop {
        let boundary = current.kind() == "closure_expression";
        if boundary {
            return None;
        }
        let function = current.kind() == "function_item";
        if function {
            break;
        }
        current = current.parent()?;
    }
    let returns_verdict = current
        .child_by_field_name("return_type")
        .is_some_and(|annotation| bool_result(annotation, source));
    let name = current.child_by_field_name("name")?;
    returns_verdict.then_some(text(name, source))
}
fn bool_result(annotation: Node<'_>, source: Code<'_>) -> bool {
    let generic = annotation.kind() == "generic_type";
    let other = !generic;
    if other {
        return false;
    }
    let result = annotation
        .child_by_field_name("type")
        .is_some_and(|path| last_segment(path, source) == "Result");
    let flag = annotation
        .child_by_field_name("type_arguments")
        .and_then(|arguments| arguments.named_child(0))
        .is_some_and(|first| first.kind() == "primitive_type" && text(first, source) == "bool");
    result && flag
}
// DECISION: D126
pub(super) fn exit_verdict(node: Node<'_>, source: Code<'_>, output: &mut Vec<Measurement>) {
    let verdict = node.kind() == "function_item"
        && node
            .child_by_field_name("return_type")
            .is_some_and(|annotation| bool_result(annotation, source));
    let Some(body) = node.child_by_field_name("body").filter(|_| verdict) else {
        return;
    };
    let coded = descendants(body)
        .into_iter()
        .any(|item| code_read(item, source));
    if coded {
        return;
    }
    let answers = VerdictBody { body, source };
    for exit in returned(body) {
        let answered = answers.success_answer(exit);
        let allowed = justified(exit, source) || in_test_context(exit, source);
        let flagged = answered && !allowed;
        if flagged {
            output.push(Measurement {
                kind: Kind::Reference(Reference::SilentError),
                line: exit.start_position().row + 1,
                symbol: "success".into(),
                actual: 1,
            });
        }
    }
}
fn code_read(node: Node<'_>, source: Code<'_>) -> bool {
    let named =
        matches!(node.kind(), "field_identifier" | "identifier") && text(node, source) == "code";
    named
        && node
            .prev_sibling()
            .is_some_and(|previous| previous.kind() == ".")
}
fn returned(body: Node<'_>) -> Vec<Node<'_>> {
    let mut cursor = body.walk();
    let tail = body
        .named_children(&mut cursor)
        .filter(|child| !child.is_extra())
        .last();
    let mut found: Vec<_> = tail
        .filter(|node| node.kind() == "call_expression")
        .into_iter()
        .collect();
    let mut pending = vec![body];
    while let Some(current) = pending.pop() {
        let nested = matches!(current.kind(), "closure_expression" | "function_item");
        if nested {
            continue;
        }
        let exit = current.kind() == "return_expression";
        if exit {
            found.push(current);
        }
        let mut cursor = current.walk();
        pending.extend(current.named_children(&mut cursor));
    }
    found
}
#[derive(Clone, Copy)]
struct VerdictBody<'a> {
    body: Node<'a>,
    source: Code<'a>,
}
impl<'a> VerdictBody<'a> {
    fn success_answer(self, exit: Node<'a>) -> bool {
        let value = match exit.kind() {
            "return_expression" => exit.named_child(0),
            _ => Some(exit),
        };
        value
            .and_then(|value| ok_argument(value, self.source))
            .and_then(|argument| method_receiver(argument, "success", self.source))
            .is_some_and(|receiver| self.process_status(receiver))
    }
    fn process_status(self, receiver: Node<'a>) -> bool {
        let receiver = self.binding(receiver).unwrap_or(receiver);
        match receiver.kind() {
            "try_expression" => launched(receiver, "status", self.source),
            "field_expression" => self.output_status(receiver),
            _ => false,
        }
    }
    fn output_status(self, receiver: Node<'a>) -> bool {
        let field = receiver
            .child_by_field_name("field")
            .is_some_and(|field| text(field, self.source) == "status");
        receiver
            .child_by_field_name("value")
            .filter(|_| field)
            .map(|value| self.binding(value).unwrap_or(value))
            .is_some_and(|value| launched(value, "output", self.source))
    }
    fn binding(self, node: Node<'a>) -> Option<Node<'a>> {
        let identifier = node.kind() == "identifier";
        let name = identifier.then(|| text(node, self.source))?;
        descendants(self.body)
            .into_iter()
            .filter(|item| item.kind() == "let_declaration")
            .filter(|item| {
                item.child_by_field_name("pattern").is_some_and(|pattern| {
                    pattern.kind() == "identifier" && text(pattern, self.source) == name
                })
            })
            .find_map(|item| item.child_by_field_name("value"))
    }
}
fn ok_argument<'a>(call: Node<'a>, source: Code<'_>) -> Option<Node<'a>> {
    let constructor = call
        .child_by_field_name("function")
        .is_some_and(|function| function.kind() == "identifier" && text(function, source) == "Ok");
    let arguments = call
        .child_by_field_name("arguments")
        .filter(|_| constructor)?;
    let mut cursor = arguments.walk();
    let values: Vec<_> = arguments
        .named_children(&mut cursor)
        .filter(|child| !child.is_extra())
        .collect();
    match values.as_slice() {
        [value] => Some(*value),
        _ => None,
    }
}
fn method_receiver<'a>(call: Node<'a>, method: &str, source: Code<'_>) -> Option<Node<'a>> {
    let bare = call.kind() == "call_expression"
        && call
            .child_by_field_name("arguments")
            .is_some_and(|arguments| arguments.named_child_count() == 0);
    let function = call.child_by_field_name("function").filter(|_| bare)?;
    let named = function
        .child_by_field_name("field")
        .is_some_and(|field| text(field, source) == method);
    function.child_by_field_name("value").filter(|_| named)
}
fn launched(node: Node<'_>, method: &str, source: Code<'_>) -> bool {
    let propagated = node.kind() == "try_expression";
    propagated
        && node
            .named_child(0)
            .and_then(|call| method_receiver(call, method, source))
            .is_some()
}
