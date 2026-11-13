use super::descriptor::{Descriptor, TEST_CODE};
use super::parameters::Parameters;
use crate::shared::diagnostics::actionable_findings::Level as Severity;

// DECISION: D177
// DECISION: D178
pub(super) const SUPERSEDEDFORM: Descriptor = Descriptor {
    default_id: "superseded-form",
    exclude: &[],
    target: "file",
    metric: "every Rust .strip_prefix(p).unwrap_or(x) or .strip_suffix(p).unwrap_or(x) call whose x, or the operand of &x, has the text of the stripped receiver, every Infallible type name, and every .collect() of a .map(f) over a parenthesized range from 0 or 0usize up to n whose turbofish or let type is Vec or that is the last expression of the body of, or the value of a return in, the nearest function or closure whose return type is Vec, test code included, outside macro arguments, without a DECISION: or INVARIANT: reference comment on the line directly above it",
    skill: "use-the-current-form",
    parameters: Parameters::Level {
        default: Severity::Error,
        violation: "the rig's Rust replaces this form with a direct standard one that states the intent in one call or name; write trim_prefix(p) or trim_suffix(p), ! or Vec::from_fn(n, f), as the finding names, or put a DECISION: or INVARIANT: reference on the line directly above it",
    },
};
// DECISION: D181
pub(super) const EPSILONTOLERANCE: Descriptor = Descriptor {
    default_id: "epsilon-tolerance",
    exclude: TEST_CODE,
    target: "file",
    metric: "every Rust path ending in f32::EPSILON or f64::EPSILON, alone, in parentheses or under unary minus, that is a direct operand of <, <=, >, >= or ==, outside test code, without a DECISION: or INVARIANT: reference comment on the line directly above the comparison",
    skill: "name-the-tolerance",
    parameters: Parameters::Level {
        default: Severity::Error,
        violation: "EPSILON is the gap between 1.0 and the next float, so as an absolute tolerance it is stricter than rounding for values above 2 and looser than the values near zero; compare with a named tolerance scaled to the magnitude and the accuracy of the computation, or put a DECISION: or INVARIANT: reference on the line directly above it",
    },
};
// DECISION: D181
pub(super) const DOTTEDEXTENSION: Descriptor = Descriptor {
    default_id: "dotted-extension",
    exclude: TEST_CODE,
    target: "file",
    metric: "every Rust with_extension or set_extension method call whose sole argument is a string literal starting with a dot, outside test code, without a DECISION: or INVARIANT: reference comment on the line directly above it",
    skill: "drop-the-dot",
    parameters: Parameters::Level {
        default: Severity::Error,
        violation: "with_extension and set_extension add the dot themselves, so an extension written with a leading dot yields a name with two dots; write the extension without the dot, or put a DECISION: or INVARIANT: reference on the line directly above it",
    },
};
