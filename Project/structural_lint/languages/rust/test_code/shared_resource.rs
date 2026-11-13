use crate::structural_lint::languages::neutral_metrics::{Code, Measurement, justified, text};
use crate::structural_lint::languages::rust::exemptions::{
    in_test_context, last_segment, zero_literal,
};
use crate::structural_lint::rule_identities::kind::Kind;
use crate::structural_lint::rule_identities::reference::Reference;
use tree_sitter::Node;

// DECISION: D181
pub(super) fn inspect(call: Node<'_>, source: Code<'_>, output: &mut Vec<Measurement>) {
    let product = !in_test_context(call, source);
    let skipped = product || justified(call, source) || !fixed_bind(call, source);
    if skipped {
        return;
    }
    output.push(Measurement {
        kind: Kind::Reference(Reference::FixedTestPort),
        line: call.start_position().row + 1,
        symbol: "bind".into(),
        actual: 1,
    });
}
fn fixed_bind(call: Node<'_>, source: Code<'_>) -> bool {
    let Some(function) = call
        .child_by_field_name("function")
        .filter(|function| function.kind() == "scoped_identifier")
    else {
        return false;
    };
    let owner = function
        .child_by_field_name("path")
        .map(|path| last_segment(path, source));
    let socket = matches!(owner, Some("TcpListener" | "UdpSocket"));
    let bound = socket && last_segment(function, source) == "bind";
    let address = call.child_by_field_name("arguments").and_then(|arguments| {
        arguments
            .named_children(&mut arguments.walk())
            .find(|argument| !argument.is_extra())
    });
    bound && address.is_some_and(|address| fixed_port(address, source))
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
