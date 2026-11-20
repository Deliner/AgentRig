use super::descriptor::{Descriptor, TEST_CODE};
use super::parameters::Parameters;
use crate::shared::diagnostics::actionable_findings::Level as Severity;

// DECISION: D177
// DECISION: D178
pub(super) const SUPERSEDEDFORM: Descriptor = Descriptor {
    default_id: "superseded-form",
    exclude: &[],
    target: "file",
    metric: "every Rust .strip_prefix(p).unwrap_or(x) or .strip_suffix(p).unwrap_or(x) call whose x, or the operand of &x, has the text of the stripped receiver, every Infallible type name, and every .collect() of a .map(f) over a parenthesized range from 0 or 0usize up to n whose turbofish or let type is Vec or that is the last expression of the body of, or the value of a return in, the nearest function or closure whose return type is Vec, test code included, outside macro arguments",
    skill: "use-the-current-form",
    parameters: Parameters::Level {
        default: Severity::Error,
        violation: "the rig's Rust replaces this form with a direct standard one that states the intent in one call or name; write trim_prefix(p) or trim_suffix(p), ! or Vec::from_fn(n, f), as the finding names",
    },
    accepts_exception: true,
};
// DECISION: D181
pub(super) const EPSILONTOLERANCE: Descriptor = Descriptor {
    default_id: "epsilon-tolerance",
    exclude: TEST_CODE,
    target: "file",
    metric: "every Rust path ending in f32::EPSILON or f64::EPSILON, alone, in parentheses or under unary minus, that is a direct operand of <, <=, >, >= or ==, outside test code, reported at the comparison",
    skill: "name-the-tolerance",
    parameters: Parameters::Level {
        default: Severity::Error,
        violation: "EPSILON is the gap between 1.0 and the next float, so as an absolute tolerance it is stricter than rounding for values above 2 and looser than the values near zero; compare with a named tolerance scaled to the magnitude and the accuracy of the computation",
    },
    accepts_exception: true,
};
// DECISION: D181
pub(super) const DOTTEDEXTENSION: Descriptor = Descriptor {
    default_id: "dotted-extension",
    exclude: TEST_CODE,
    target: "file",
    metric: "every Rust with_extension or set_extension method call whose sole argument is a string literal starting with a dot, outside test code",
    skill: "drop-the-dot",
    parameters: Parameters::Level {
        default: Severity::Error,
        violation: "with_extension and set_extension add the dot themselves, so an extension written with a leading dot yields a name with two dots; write the extension without the dot",
    },
    accepts_exception: true,
};
// DECISION: D205
pub(super) const INDEXEDARRAYLITERAL: Descriptor = Descriptor {
    default_id: "indexed-array-literal",
    exclude: &[],
    target: "file",
    metric: "every Rust array literal that lists its elements, at least one of which contains a call, indexed by an expression other than an integer literal or the full range .., test code included",
    skill: "branch-on-the-index",
    parameters: Parameters::Level {
        default: Severity::Error,
        violation: "indexing an array literal evaluates every element and keeps only the one the index selects, so the calls of the others run for nothing; select with match or if so that only the chosen element is computed",
    },
    accepts_exception: true,
};
// DECISION: D208
pub(super) const OUTERFLAGFILTER: Descriptor = Descriptor {
    default_id: "outer-flag-filter",
    exclude: &[],
    target: "file",
    metric: "every Rust .filter(..) method call whose receiver contains a call and whose sole argument is a closure with the single parameter _ and a body without a call, test code included",
    skill: "check-the-flag-first",
    parameters: Parameters::Level {
        default: Severity::Error,
        violation: "a filter that ignores its value keeps or drops the whole result by a flag known before the call, so the call runs even when the flag throws its result away; check the flag first with if, or with flag.then(..) before the call",
    },
    accepts_exception: true,
};
// DECISION: D209
pub(super) const MAXFALLBACK: Descriptor = Descriptor {
    default_id: "max-fallback",
    exclude: TEST_CODE,
    target: "file",
    metric: "every Rust binary + or * whose left or right operand is a .unwrap_or(..) method call whose sole argument is MAX or a path ending in ::MAX, outside test code",
    skill: "saturate-or-handle",
    parameters: Parameters::Level {
        default: Severity::Error,
        violation: "a MAX fallback stands for a value that is missing or does not fit, and adding to it or multiplying it overflows, panicking in debug builds and wrapping in release, or yields a number that no longer means the bound; handle the missing value before the arithmetic or compute with saturating_add or saturating_mul",
    },
    accepts_exception: true,
};
