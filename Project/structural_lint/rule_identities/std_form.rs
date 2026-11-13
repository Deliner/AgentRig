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
pub(super) const REMAINDERSIGN: Descriptor = Descriptor {
    default_id: "remainder-sign",
    exclude: TEST_CODE,
    target: "file",
    metric: "every Rust % expression, alone or in parentheses, that is a direct operand of == or != whose other operand is a nonzero integer or float literal, or of <, <=, > or >= whose other operand is an integer or float literal of any value, outside test code, without a DECISION: or INVARIANT: reference comment on the line directly above the comparison",
    skill: "use-euclidean-remainder",
    parameters: Parameters::Level {
        default: Severity::Error,
        violation: "the remainder of a negative dividend is negative or zero in Rust, so a comparison with a positive value or an ordering against a bound misses every negative case; use rem_euclid for a remainder in 0..n, compare the remainder with 0, or put a DECISION: or INVARIANT: reference on the line directly above it",
    },
};
pub(super) const PARTIALORDERFALLBACK: Descriptor = Descriptor {
    default_id: "partial-order-fallback",
    exclude: TEST_CODE,
    target: "file",
    metric: "every Rust .unwrap_or(a) or .unwrap_or_else(a) call directly on a .partial_cmp(..) call where a contains the name Equal, Less or Greater, outside test code, without a DECISION: or INVARIANT: reference comment on the line directly above it",
    skill: "order-floats-totally",
    parameters: Parameters::Level {
        default: Severity::Error,
        violation: "partial_cmp returns None for NaN, and a fixed Ordering in its place makes the order inconsistent, so a sort or search can panic or silently misplace values; compare with total_cmp, reject or filter NaN before comparing, or put a DECISION: or INVARIANT: reference on the line directly above it",
    },
};
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
pub(super) const UNSTABLEVALUE: Descriptor = Descriptor {
    default_id: "unstable-value",
    exclude: TEST_CODE,
    target: "file",
    metric: "every Rust DefaultHasher that is a segment of an expression path outside a use declaration, every call without arguments of a path ending in type_name and every call of a path ending in type_name_of_val, outside test code, without a DECISION: or INVARIANT: reference comment on the line directly above it",
    skill: "pin-the-value",
    parameters: Parameters::Level {
        default: Severity::Error,
        violation: "the standard library does not specify the DefaultHasher algorithm or the text of type_name, and either may change between Rust releases, so a value stored, sent or ordered by them changes with the toolchain; use a hasher with a fixed algorithm and seed or a name the code defines, or put a DECISION: or INVARIANT: reference on the line directly above it",
    },
};
pub(super) const UNSTABLESORT: Descriptor = Descriptor {
    default_id: "unstable-sort",
    exclude: TEST_CODE,
    target: "file",
    metric: "every Rust sort_unstable_by or sort_unstable_by_key method call, outside test code, without a DECISION: or INVARIANT: reference comment on the line directly above it",
    skill: "sort-stably",
    parameters: Parameters::Level {
        default: Severity::Error,
        violation: "an unstable sort may reorder elements that compare equal by the comparator or key, so ties come out in an order nobody specified, which can change with the input order or the Rust release; use sort_by or sort_by_key, extend the key until it is unique, or put a DECISION: or INVARIANT: reference on the line directly above it",
    },
};
pub(super) const FRONTREMOVAL: Descriptor = Descriptor {
    default_id: "front-removal",
    exclude: TEST_CODE,
    target: "file",
    metric: "every Rust remove method call whose sole argument is the integer literal 0, outside test code, without a DECISION: or INVARIANT: reference comment on the line directly above it",
    skill: "pop-the-front",
    parameters: Parameters::Level {
        default: Severity::Error,
        violation: "remove(0) on a Vec or String shifts every remaining element and panics when it is empty, so draining from the front takes quadratic time; use a VecDeque with pop_front, iterate or drain instead, or put a DECISION: or INVARIANT: reference on the line directly above it",
    },
};
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
