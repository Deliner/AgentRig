use super::Analysis;
use super::neutral_metrics::{Measurement, justified_line};
use crate::structural_lint::rule_identities::kind::Kind;
use crate::structural_lint::rule_identities::reference::Reference;
use yaml_rust2::parser::{Event, Parser};

// DECISION: D183
const LOWERING: [&str; 4] = ["-A", "--allow", "--cap-lints", "--force-warn"];

#[derive(Clone, Copy)]
enum List {
    Flags,
    Command,
}

struct Sequence {
    list: List,
    items: Vec<(usize, String)>,
}

enum Slot {
    Key,
    Value,
    ListValue(List),
    Members(Sequence),
    Items,
}

enum Role<'a> {
    ListValue(List),
    Member(&'a mut Sequence),
    Other,
}

// DECISION: D182
pub(super) fn analyze(source: &str) -> Analysis {
    let mut analysis = Analysis::default();
    let mut parser = Parser::new_from_str(source);
    let mut slots = Vec::new();
    let mut sequences = Vec::new();
    loop {
        match parser.next_token() {
            Err(error) => {
                analysis.parse_error = Some(error.marker().line());
                break;
            }
            Ok((Event::StreamEnd, _)) => break,
            Ok((event, mark)) => sequences.extend(step(event, mark.line(), &mut slots)),
        }
    }
    analysis.measurements = sequences
        .into_iter()
        .flat_map(checked_items)
        .filter(|(line, flag)| lowering(flag) && !justified_line(line - 1, source))
        .map(|(line, symbol)| Measurement {
            kind: Kind::Reference(Reference::AllowJustification),
            line,
            symbol,
            actual: 1,
        })
        .collect();
    analysis
}
fn step(event: Event, line: usize, slots: &mut Vec<Slot>) -> Option<Sequence> {
    match event {
        Event::Scalar(text, ..) => {
            if let Role::Member(sequence) = advance(slots, &text) {
                sequence.items.push((line, text));
            }
            None
        }
        Event::SequenceStart(..) => {
            let slot = match advance(slots, "") {
                Role::ListValue(list) => Slot::Members(Sequence {
                    list,
                    items: Vec::new(),
                }),
                Role::Member(_) | Role::Other => Slot::Items,
            };
            slots.push(slot);
            None
        }
        Event::MappingStart(..) => {
            advance(slots, "");
            slots.push(Slot::Key);
            None
        }
        Event::Alias(_) => {
            advance(slots, "");
            None
        }
        Event::SequenceEnd | Event::MappingEnd => match slots.pop() {
            Some(Slot::Members(sequence)) => Some(sequence),
            Some(Slot::Key | Slot::Value | Slot::ListValue(_) | Slot::Items) | None => None,
        },
        Event::Nothing
        | Event::StreamStart
        | Event::StreamEnd
        | Event::DocumentStart
        | Event::DocumentEnd => None,
    }
}
fn advance<'a>(slots: &'a mut [Slot], scalar: &str) -> Role<'a> {
    let Some(slot) = slots.last_mut() else {
        return Role::Other;
    };
    match slot {
        Slot::Key => {
            *slot = match scalar {
                "rustflags" => Slot::ListValue(List::Flags),
                "argv" => Slot::ListValue(List::Command),
                _ => Slot::Value,
            };
            Role::Other
        }
        Slot::Value => {
            *slot = Slot::Key;
            Role::Other
        }
        Slot::ListValue(list) => {
            let list = *list;
            *slot = Slot::Key;
            Role::ListValue(list)
        }
        Slot::Members(sequence) => Role::Member(sequence),
        Slot::Items => Role::Other,
    }
}
fn checked_items(sequence: Sequence) -> Vec<(usize, String)> {
    match sequence.list {
        List::Flags => sequence.items,
        List::Command => compiler_arguments(sequence.items),
    }
}
// DECISION: D184
fn compiler_arguments(mut argv: Vec<(usize, String)>) -> Vec<(usize, String)> {
    let separator = argv.iter().position(|(_, item)| item == "--");
    let arguments = separator
        .map(|index| argv.split_off(index + 1))
        .unwrap_or_default();
    let cargo = argv.first().is_some_and(|(_, program)| program == "cargo");
    let compiler = argv
        .iter()
        .any(|(_, item)| matches!(item.as_str(), "clippy" | "rustc"));
    let compiler_command = cargo && compiler;
    if compiler_command {
        arguments
    } else {
        Vec::new()
    }
}
fn lowering(flag: &str) -> bool {
    LOWERING.iter().any(|prefix| flag.starts_with(prefix))
}
