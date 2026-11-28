use super::neutral_metrics::{Analysis, Measurement};
use crate::structural_lint::rule_identities::{Kind, Reference};
use yaml_rust2::parser::{Event, Parser};

// DECISION: D183
const LOWERING: [&str; 4] = ["-A", "--allow", "--cap-lints", "--force-warn"];

enum Slot {
    Key,
    Value,
    ListValue,
    Members(Vec<(usize, String)>),
    Items,
}

enum Role<'a> {
    ListValue,
    Member(&'a mut Vec<(usize, String)>),
    Other,
}

// DECISION: D182
pub(super) fn analyze(source: &str) -> Analysis {
    let mut analysis = Analysis::default();
    let mut parser = Parser::new_from_str(source);
    let mut slots = Vec::new();
    let mut lists = Vec::new();
    loop {
        match parser.next_token() {
            Err(error) => {
                analysis.parse_error = Some(error.marker().line());
                break;
            }
            Ok((Event::StreamEnd, _)) => break,
            Ok((event, mark)) => lists.extend(step(event, mark.line(), &mut slots)),
        }
    }
    analysis.measurements = lists
        .into_iter()
        .flatten()
        .filter(|(_, flag)| lowering(flag))
        .map(|(line, symbol)| Measurement {
            kind: Kind::Reference(Reference::AllowJustification),
            line,
            symbol,
            actual: 1,
        })
        .collect();
    analysis
}
fn step(event: Event, line: usize, slots: &mut Vec<Slot>) -> Option<Vec<(usize, String)>> {
    match event {
        Event::Scalar(text, ..) => {
            if let Role::Member(items) = advance(slots, &text) {
                items.push((line, text));
            }
            None
        }
        Event::SequenceStart(..) => {
            let slot = match advance(slots, "") {
                Role::ListValue => Slot::Members(Vec::new()),
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
            Some(Slot::Members(items)) => Some(items),
            Some(Slot::Key | Slot::Value | Slot::ListValue | Slot::Items) | None => None,
        },
        Event::Nothing
        | Event::StreamStart
        | Event::StreamEnd
        | Event::DocumentStart
        | Event::DocumentEnd => None,
    }
}
// DECISION: D239
fn advance<'a>(slots: &'a mut [Slot], scalar: &str) -> Role<'a> {
    let Some(slot) = slots.last_mut() else {
        return Role::Other;
    };
    match slot {
        Slot::Key => {
            *slot = match scalar {
                "rustflags" => Slot::ListValue,
                _ => Slot::Value,
            };
            Role::Other
        }
        Slot::Value => {
            *slot = Slot::Key;
            Role::Other
        }
        Slot::ListValue => {
            *slot = Slot::Key;
            Role::ListValue
        }
        Slot::Members(items) => Role::Member(items),
        Slot::Items => Role::Other,
    }
}
fn lowering(flag: &str) -> bool {
    LOWERING.iter().any(|prefix| flag.starts_with(prefix))
}
