use super::exemptions::{in_test_context, last_segment};
use crate::structural_lint::languages::neutral_metrics::{Code, Measurement, justified, text};
use crate::structural_lint::rule_identities::kind::Kind;
use crate::structural_lint::rule_identities::reference::Reference;
use tree_sitter::Node;

const SHELLS: &[&str] = &[
    "sh",
    "bash",
    "zsh",
    "dash",
    "ksh",
    "fish",
    "cmd",
    "powershell",
    "pwsh",
];
const ESCAPES: &[(&str, &str)] = &[("n", "\n"), ("t", "\t"), ("r", "\r"), ("0", "\0")];

// DECISION: D170
pub(super) fn inspect(call: Node<'_>, source: Code<'_>, output: &mut Vec<Measurement>) {
    let Some(shell) = shell_name(call, source) else {
        return;
    };
    let passed = chained_arguments(call, source);
    let interpolated = passed.windows(2).any(
        |pair| matches!(pair, [flag, value] if command_flag(*flag, source) && data(*value, source)),
    );
    let other = !interpolated;
    if other {
        return;
    }
    let allowed = justified(call, source) || in_test_context(call, source);
    if allowed {
        return;
    }
    output.push(Measurement {
        kind: Kind::Reference(Reference::InterpolatedShell),
        line: call.start_position().row + 1,
        symbol: shell,
        actual: 1,
    });
}
fn shell_name(call: Node<'_>, source: Code<'_>) -> Option<String> {
    let function = call.child_by_field_name("function")?;
    let scoped = function.kind() == "scoped_identifier";
    let constructor = scoped && last_segment(function, source) == "new";
    let command = function
        .child_by_field_name("path")
        .is_some_and(|path| last_segment(path, source) == "Command");
    let spawning = constructor && command;
    let other = !spawning;
    if other {
        return None;
    }
    let arguments = expressions(call.child_by_field_name("arguments")?);
    let value = literal_value(*arguments.first()?, source)?;
    let file = value.rsplit(['/', '\\']).next()?.to_ascii_lowercase();
    let name = file.trim_suffix(".exe");
    SHELLS.contains(&name).then(|| name.to_string())
}
fn chained_arguments<'a>(call: Node<'a>, source: Code<'_>) -> Vec<Node<'a>> {
    let mut passed = Vec::new();
    let mut receiver = call;
    while let Some(access) = receiver
        .parent()
        .filter(|access| access.kind() == "field_expression")
    {
        let Some(chained) = access
            .parent()
            .filter(|parent| parent.kind() == "call_expression")
        else {
            break;
        };
        let name = access
            .child_by_field_name("field")
            .map_or("", |field| text(field, source));
        let arguments = chained
            .child_by_field_name("arguments")
            .map_or_else(Vec::new, expressions);
        let listed = match name {
            "arg" => arguments,
            "args" => listed_array(&arguments),
            _ => Vec::new(),
        };
        let opaque = name == "args" && listed.is_empty();
        if opaque {
            break;
        }
        passed.extend(listed);
        receiver = chained;
    }
    passed
}
fn listed_array<'a>(arguments: &[Node<'a>]) -> Vec<Node<'a>> {
    arguments
        .first()
        .map(|argument| unreferenced(*argument))
        .filter(|array| array.kind() == "array_expression")
        .map_or_else(Vec::new, expressions)
}
fn command_flag(argument: Node<'_>, source: Code<'_>) -> bool {
    let Some(flag) = literal_value(unreferenced(argument), source) else {
        return false;
    };
    let clustered = flag.strip_prefix('-').is_some_and(|letters| {
        letters.ends_with('c') && letters.chars().all(|letter| letter.is_ascii_lowercase())
    });
    let spelled = matches!(flag.to_ascii_lowercase().as_str(), "/c" | "-command");
    clustered || spelled
}
fn data(argument: Node<'_>, source: Code<'_>) -> bool {
    let value = unreferenced(argument);
    let literal = matches!(value.kind(), "string_literal" | "raw_string_literal");
    let named = matches!(value.kind(), "identifier" | "scoped_identifier");
    let constant = named && constant_name(last_segment(value, source));
    let fixed = literal || constant;
    !fixed
}
fn constant_name(name: &str) -> bool {
    let upper = name.chars().any(|letter| letter.is_ascii_uppercase());
    upper
        && name
            .chars()
            .all(|letter| matches!(letter, 'A'..='Z' | '0'..='9' | '_'))
}
fn unreferenced(node: Node<'_>) -> Node<'_> {
    let referenced = node.kind() == "reference_expression";
    match node.child_by_field_name("value") {
        Some(value) if referenced => unreferenced(value),
        _ => node,
    }
}
fn expressions(list: Node<'_>) -> Vec<Node<'_>> {
    let mut cursor = list.walk();
    list.named_children(&mut cursor)
        .filter(|child| !child.kind().ends_with("comment"))
        .collect()
}
fn literal_value(node: Node<'_>, source: Code<'_>) -> Option<String> {
    let literal = matches!(node.kind(), "string_literal" | "raw_string_literal");
    let other = !literal;
    if other {
        return None;
    }
    let mut cursor = node.walk();
    let value: String = node
        .named_children(&mut cursor)
        .map(|part| match part.kind() {
            "escape_sequence" => escaped(text(part, source)),
            _ => text(part, source),
        })
        .collect();
    Some(value)
}
fn escaped(sequence: &str) -> &str {
    let rest = sequence.trim_prefix('\\');
    let continued = rest.starts_with(char::is_whitespace);
    if continued {
        return "";
    }
    let named = ESCAPES
        .iter()
        .find(|(name, _)| *name == rest)
        .map(|(_, value)| *value);
    let quoted = (rest.chars().count() == 1).then_some(rest);
    named.or(quoted).unwrap_or(sequence)
}
