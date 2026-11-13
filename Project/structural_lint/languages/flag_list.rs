use super::Analysis;
use super::neutral_metrics::{Measurement, justified_line};
use crate::structural_lint::rule_identities::kind::Kind;
use crate::structural_lint::rule_identities::reference::Reference;
use yaml_rust2::parser::{Event, Parser};

const SILENCING: [&str; 3] = ["-A", "--allow", "--cap-lints"];

#[derive(Clone, Copy)]
enum Slot {
    Key,
    Value,
    FlagValue,
    Flags,
    Items,
}

// DECISION: D182
pub(super) fn analyze(source: &str) -> Analysis {
    let mut analysis = Analysis::default();
    let mut parser = Parser::new_from_str(source);
    let mut slots = Vec::new();
    let mut items = Vec::new();
    loop {
        match parser.next_token() {
            Err(error) => {
                analysis.parse_error = Some(error.marker().line());
                break;
            }
            Ok((Event::StreamEnd, _)) => break,
            Ok((event, mark)) => {
                items.extend(item(event, &mut slots).map(|flag| (mark.line(), flag)))
            }
        }
    }
    analysis.measurements = items
        .into_iter()
        .filter(|(line, flag)| silencing(flag) && !justified_line(line - 1, source))
        .map(|(line, symbol)| Measurement {
            kind: Kind::Reference(Reference::AllowJustification),
            line,
            symbol,
            actual: 1,
        })
        .collect();
    analysis
}
fn item(event: Event, slots: &mut Vec<Slot>) -> Option<String> {
    match event {
        Event::Scalar(text, ..) => match advance(slots, &text) {
            Some(Slot::Flags) => Some(text),
            Some(Slot::Key | Slot::Value | Slot::FlagValue | Slot::Items) | None => None,
        },
        Event::SequenceStart(..) => {
            let slot = match advance(slots, "") {
                Some(Slot::FlagValue) => Slot::Flags,
                Some(Slot::Key | Slot::Value | Slot::Flags | Slot::Items) | None => Slot::Items,
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
        Event::SequenceEnd | Event::MappingEnd => {
            slots.pop();
            None
        }
        Event::Nothing
        | Event::StreamStart
        | Event::StreamEnd
        | Event::DocumentStart
        | Event::DocumentEnd => None,
    }
}
fn advance(slots: &mut [Slot], scalar: &str) -> Option<Slot> {
    let slot = slots.last_mut()?;
    let previous = *slot;
    *slot = match previous {
        Slot::Key if scalar == "rustflags" => Slot::FlagValue,
        Slot::Key => Slot::Value,
        Slot::Value | Slot::FlagValue => Slot::Key,
        Slot::Flags | Slot::Items => previous,
    };
    Some(previous)
}
fn silencing(flag: &str) -> bool {
    SILENCING.iter().any(|prefix| flag.starts_with(prefix))
}
