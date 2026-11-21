use super::call_syntax::method_receiver;
use super::error_verdict::{bool_result, ok_argument};
use crate::structural_lint::languages::neutral_metrics::{Code, Measurement, descendants, text};
use crate::structural_lint::languages::rust::exemptions::in_test_context;
use crate::structural_lint::rule_identities::{Kind, Reference};
use tree_sitter::Node;

// DECISION: D126
pub(super) fn inspect(node: Node<'_>, source: Code<'_>, output: &mut Vec<Measurement>) {
    let verdict = node.kind() == "function_item"
        && node
            .child_by_field_name("return_type")
            .is_some_and(|annotation| bool_result(annotation, source));
    let Some(body) = verdict.then(|| node.child_by_field_name("body")).flatten() else {
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
        let allowed = in_test_context(exit, source);
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
        field
            .then(|| receiver.child_by_field_name("value"))
            .flatten()
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
fn launched(node: Node<'_>, method: &str, source: Code<'_>) -> bool {
    let propagated = node.kind() == "try_expression";
    propagated
        && node
            .named_child(0)
            .and_then(|call| method_receiver(call, method, source))
            .is_some()
}
