use crate::structural_lint::languages::neutral_metrics::{Code, Measurement, justified, text};
use crate::structural_lint::languages::rust::exemptions::{
    in_test_context, last_segment, zero_literal,
};
use crate::structural_lint::rule_identities::kind::Kind;
use crate::structural_lint::rule_identities::reference::Reference;
use tree_sitter::Node;

pub(super) fn inspect(call: Node<'_>, source: Code<'_>, output: &mut Vec<Measurement>) {
    let Some((reference, name)) = shared(call, source) else {
        return;
    };
    let product = !in_test_context(call, source);
    let allowed = product || justified(call, source);
    if allowed {
        return;
    }
    output.push(Measurement {
        kind: Kind::Reference(reference),
        line: call.start_position().row + 1,
        symbol: name.into(),
        actual: 1,
    });
}
fn shared<'a>(call: Node<'_>, source: Code<'a>) -> Option<(Reference, &'a str)> {
    let function = call
        .child_by_field_name("function")
        .filter(|function| function.kind() == "scoped_identifier")?;
    let name = last_segment(function, source);
    let owner = last_segment(function.child_by_field_name("path")?, source);
    let arguments = call.child_by_field_name("arguments")?;
    let first = arguments
        .named_children(&mut arguments.walk())
        .find(|argument| !argument.is_extra())?;
    let literal = matches!(first.kind(), "string_literal" | "raw_string_literal");
    match (owner, name) {
        ("TcpListener" | "UdpSocket", "bind") => {
            fixed_port(first, source).then_some((Reference::FixedTestPort, name))
        }
        ("File", "create") | ("fs", "write" | "create_dir" | "create_dir_all") => {
            literal.then_some((Reference::FixedTestPath, name))
        }
        _ => None,
    }
}
fn fixed_port(address: Node<'_>, source: Code<'_>) -> bool {
    match address.kind() {
        "string_literal" | "raw_string_literal" => address
            .named_child(0)
            .and_then(|content| text(content, source).rsplit_once(':'))
            .is_some_and(|(_, port)| port.parse::<u16>().is_ok_and(|port| port != 0)),
        "tuple_expression" => address
            .named_children(&mut address.walk())
            .filter(|part| !part.is_extra())
            .nth(1)
            .is_some_and(|port| port.kind() == "integer_literal" && !zero_literal(port, source)),
        _ => false,
    }
}
