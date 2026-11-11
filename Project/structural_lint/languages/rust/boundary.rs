use super::context::{in_test_context, last_segment};
use super::naming::template;
use crate::structural_lint::languages::measure::{Code, Measurement, descendants, justified, text};
use crate::structural_lint::rules::kind::Kind;
use crate::structural_lint::rules::reference::Reference;
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
const BUILDERS: &[&str] = &["format", "write", "writeln"];
const STATEMENTS: &[&str] = &["SELECT", "INSERT", "UPDATE", "DELETE"];
const CHECKS: &[&str] = &["exists", "is_file", "is_dir"];
const ESCAPES: &[(&str, &str)] = &[("n", "\n"), ("t", "\t"), ("r", "\r"), ("0", "\0")];

pub(super) fn inspect(node: Node<'_>, source: Code<'_>, output: &mut Vec<Measurement>) {
    let found = match node.kind() {
        "call_expression" => interpolated_shell(node, source).or_else(|| stale_check(node, source)),
        "macro_invocation" => interpolated_syntax(node, source),
        _ => None,
    };
    let Some((reference, symbol)) = found else {
        return;
    };
    let allowed = justified(node, source) || in_test_context(node, source);
    if allowed {
        return;
    }
    output.push(Measurement {
        kind: Kind::Reference(reference),
        line: node.start_position().row + 1,
        symbol,
        actual: 1,
    });
}
fn interpolated_shell(call: Node<'_>, source: Code<'_>) -> Option<(Reference, String)> {
    let shell = shell_name(call, source)?;
    let passed = chained_arguments(call, source);
    let interpolated = passed.windows(2).any(
        |pair| matches!(pair, [flag, value] if command_flag(*flag, source) && data(*value, source)),
    );
    interpolated.then_some((Reference::InterpolatedShell, shell))
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
    let name = file.strip_suffix(".exe").unwrap_or(&file);
    SHELLS.contains(&name).then(|| name.to_string())
}
fn chained_arguments<'a>(call: Node<'a>, source: Code<'_>) -> Vec<Node<'a>> {
    let mut passed = Vec::new();
    let mut receiver = call;
    while let Some(chained) = receiver
        .parent()
        .filter(|access| access.kind() == "field_expression")
        .and_then(|access| access.parent())
        .filter(|parent| parent.kind() == "call_expression")
    {
        let Some(link) = method(chained, source) else {
            break;
        };
        let listed = match link.name {
            "arg" => link.arguments,
            "args" => listed_array(&link.arguments, source),
            _ => Vec::new(),
        };
        let opaque = link.name == "args" && listed.is_empty();
        if opaque {
            break;
        }
        passed.extend(listed);
        receiver = chained;
    }
    passed
}
fn listed_array<'a>(arguments: &[Node<'a>], source: Code<'_>) -> Vec<Node<'a>> {
    arguments
        .first()
        .map(|argument| stripped(*argument, source))
        .filter(|array| array.kind() == "array_expression")
        .map_or_else(Vec::new, expressions)
}
fn command_flag(argument: Node<'_>, source: Code<'_>) -> bool {
    let Some(flag) = literal_value(stripped(argument, source), source) else {
        return false;
    };
    let clustered = flag.strip_prefix('-').is_some_and(|letters| {
        letters.ends_with('c') && letters.chars().all(|letter| letter.is_ascii_lowercase())
    });
    let spelled = matches!(flag.to_ascii_lowercase().as_str(), "/c" | "-command");
    clustered || spelled
}
fn data(argument: Node<'_>, source: Code<'_>) -> bool {
    let value = stripped(argument, source);
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
fn interpolated_syntax(invocation: Node<'_>, source: Code<'_>) -> Option<(Reference, String)> {
    let name = invocation
        .child_by_field_name("macro")
        .map(|path| last_segment(path, source))
        .filter(|name| BUILDERS.contains(name))?;
    let value = literal_value(template(invocation)?, source)?;
    let structured = json_object(&value) || sql_statement(&value);
    let interpolated = structured && value.replace("{{", "").contains('{');
    interpolated.then(|| (Reference::InterpolatedSyntax, name.to_string()))
}
fn json_object(template: &str) -> bool {
    template
        .split("{{")
        .skip(1)
        .any(|rest| rest.trim_start().starts_with('"'))
}
fn sql_statement(template: &str) -> bool {
    let trimmed = template.trim_start();
    STATEMENTS.iter().any(|keyword| {
        trimmed
            .strip_prefix(keyword)
            .is_some_and(|rest| rest.starts_with(char::is_whitespace))
    })
}
fn stale_check(call: Node<'_>, source: Code<'_>) -> Option<(Reference, String)> {
    let checked = method(call, source)?;
    let checking = checked.arguments.is_empty() && CHECKS.contains(&checked.name);
    let other = !checking;
    if other {
        return None;
    }
    let path = text(stripped(checked.receiver, source), source);
    let scope = enclosing_function(call)?;
    let acted = descendants(scope)
        .into_iter()
        .filter(|later| later.kind() == "call_expression" && later.start_byte() >= call.end_byte())
        .any(|later| enclosing_function(later) == Some(scope) && acts_on(later, path, source));
    acted.then(|| (Reference::StaleCheck, checked.name.to_string()))
}
fn acts_on(call: Node<'_>, path: &str, source: Code<'_>) -> bool {
    let function = call
        .child_by_field_name("function")
        .filter(|function| function.kind() == "scoped_identifier");
    let Some(function) = function else {
        return false;
    };
    let segments: Vec<&str> = text(function, source).split("::").map(str::trim).collect();
    let filesystem = segments
        .iter()
        .rev()
        .skip(1)
        .any(|segment| *segment == "fs");
    let opened = matches!(segments.as_slice(), [.., "File", "open" | "create"]);
    let acting = filesystem || opened;
    let arguments = call
        .child_by_field_name("arguments")
        .map_or_else(Vec::new, expressions);
    acting
        && arguments
            .into_iter()
            .any(|argument| text(stripped(argument, source), source) == path)
}
struct MethodCall<'tree, 'text> {
    name: &'text str,
    receiver: Node<'tree>,
    arguments: Vec<Node<'tree>>,
}
fn method<'tree, 'text>(
    call: Node<'tree>,
    source: Code<'text>,
) -> Option<MethodCall<'tree, 'text>> {
    let access = call
        .child_by_field_name("function")
        .filter(|function| function.kind() == "field_expression")?;
    Some(MethodCall {
        name: text(access.child_by_field_name("field")?, source),
        receiver: access.child_by_field_name("value")?,
        arguments: expressions(call.child_by_field_name("arguments")?),
    })
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
        "call_expression" => method(node, source)
            .filter(|call| call.name == "as_path" && call.arguments.is_empty())
            .map(|call| call.receiver),
        _ => None,
    }
}
fn enclosing_function(node: Node<'_>) -> Option<Node<'_>> {
    std::iter::successors(node.parent(), |ancestor| ancestor.parent())
        .find(|ancestor| ancestor.kind() == "function_item")
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
    let value = node
        .named_children(&mut cursor)
        .map(|part| match part.kind() {
            "escape_sequence" => escaped(text(part, source)),
            _ => text(part, source),
        })
        .collect();
    Some(value)
}
fn escaped(sequence: &str) -> &str {
    let rest = sequence.strip_prefix('\\').unwrap_or(sequence);
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
