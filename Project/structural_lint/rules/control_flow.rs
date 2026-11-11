use super::descriptor::{Descriptor, TEST_CODE};
use super::parameters::Parameters;
use crate::shared::diagnostics::lint::Level as Severity;
use serde::{Deserialize, Serialize};

// DECISION: D133
#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum ControlFlow {
    NamedIfCondition,
    NestingDepth,
    BranchBudget,
    BooleanExpression,
    UnlabeledExit,
    LengthSubtraction,
    VariantEquality,
    RangeChain,
    RepeatedFragment,
    HiddenExit,
}
impl ControlFlow {
    pub(super) fn descriptor(self) -> &'static Descriptor {
        match self {
            Self::NamedIfCondition => &NAMEDIFCONDITION,
            Self::NestingDepth => &NESTINGDEPTH,
            Self::BranchBudget => &BRANCHBUDGET,
            Self::BooleanExpression => &BOOLEANEXPRESSION,
            Self::UnlabeledExit => &UNLABELEDEXIT,
            Self::LengthSubtraction => &LENGTHSUBTRACTION,
            Self::VariantEquality => &VARIANTEQUALITY,
            Self::RangeChain => &RANGECHAIN,
            Self::RepeatedFragment => &REPEATEDFRAGMENT,
            Self::HiddenExit => &HIDDENEXIT,
        }
    }
}
// DECISION: D156
#[derive(Default, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Junctions {
    #[default]
    Mixed,
    Single,
}
impl Junctions {
    pub fn mixed(&self) -> bool {
        match self {
            Self::Mixed => true,
            Self::Single => false,
        }
    }
}
pub const MIXED_JUNCTIONS: &str = "one boolean expression joins conjunctions with disjunctions, and parentheses do not end it, so a reader must work out how its junctions nest; bind each conjunction or disjunction to a named value and combine the names with one junction";
// DECISION: D156
const VARIANTEQUALITY: Descriptor = Descriptor {
    default_id: "variant-equality",
    exclude: TEST_CODE,
    target: "file",
    metric: "every Rust == or != with an operand that is a path of two or more segments ending in an UpperCamelCase name, and every matches! whose pattern names such a path, outside test code, without a DECISION: or INVARIANT: reference comment on the line directly above it",
    skill: "match-every-variant",
    parameters: Parameters::Level {
        default: Severity::Error,
        violation: "comparing with one variant sends every variant added later to the other branch silently; match the enum exhaustively so the compiler names each new variant, or put a DECISION: or INVARIANT: reference on the line directly above it",
    },
};
// DECISION: D156
const RANGECHAIN: Descriptor = Descriptor {
    default_id: "range-chain",
    exclude: TEST_CODE,
    target: "file",
    metric: "every Rust if and else-if chain of two or more conditions, and every match with two or more guards, comparing one token-equal expression with an integer literal by <, <=, > or >=, outside test code, without a DECISION: or INVARIANT: reference comment on the line directly above it",
    skill: "match-the-ranges",
    parameters: Parameters::Level {
        default: Severity::Error,
        violation: "a chain of comparisons hides a gap between bounds and a branch made unreachable by order; match on range patterns so the compiler checks coverage, or put a DECISION: or INVARIANT: reference on the line directly above it",
    },
};
const NAMEDIFCONDITION: Descriptor = Descriptor {
    default_id: "named-if",
    exclude: &[],
    target: "file",
    metric: "one named value per boolean if condition; Rust if-let bindings excluded; named field paths accepted",
    skill: "name-if-condition",
    parameters: Parameters::Level {
        default: Severity::Error,
        violation: "if condition must be one named value; give the branch reason a name",
    },
};
// DECISION: D061
const NESTINGDEPTH: Descriptor = Descriptor {
    default_id: "nesting-depth",
    exclude: &[],
    target: "file",
    metric: "deepest nested control-flow blocks in a function; if/match/loop/while/for add depth; Rust else-if and Python elif continue the same level; nested function and closure bodies are measured on their own",
    skill: "flatten-nesting",
    parameters: Parameters::Thresholds {
        warning: None,
        error: 4,
    },
};
// DECISION: D063
const BRANCHBUDGET: Descriptor = Descriptor {
    default_id: "branch-budget",
    exclude: &[],
    target: "file",
    metric: "decision points in a function: each if, else-if and ternary condition, and each match or case as one multi-way decision regardless of arms; nested function and closure bodies are measured on their own",
    skill: "reduce-branching",
    parameters: Parameters::Thresholds {
        warning: None,
        error: 5,
    },
};
// DECISION: D120
const BOOLEANEXPRESSION: Descriptor = Descriptor {
    default_id: "boolean-expression",
    exclude: &[],
    target: "file",
    metric: "operands of one Rust && and || or Python and and or expression anywhere in code: parentheses group operands without ending the expression, a let binding of a Rust let chain is not an operand, and a negation, comparison, call, closure or macro is one operand whose inner expressions are measured on their own; with junctions single, also every such expression that joins && with || or and with or, directly or inside parentheses, at error level",
    skill: "split-boolean-expression",
    parameters: Parameters::Thresholds {
        warning: None,
        error: 3,
    },
};
// DECISION: D155
const UNLABELEDEXIT: Descriptor = Descriptor {
    default_id: "unlabeled-exit",
    exclude: TEST_CODE,
    target: "file",
    metric: "every Rust break or continue without a label whose nearest loop lies inside another loop of the same function, closures and async blocks bounding both searches, outside test code, without a DECISION: or INVARIANT: reference comment on the line directly above it",
    skill: "label-the-exit",
    parameters: Parameters::Level {
        default: Severity::Error,
        violation: "an unlabeled break or continue in a nested loop acts on the innermost loop, which a reader or a later edit easily mistakes for the outer one; label the loop it targets and name the label in the exit",
    },
};
// DECISION: D155
const LENGTHSUBTRACTION: Descriptor = Descriptor {
    default_id: "length-subtraction",
    exclude: TEST_CODE,
    target: "file",
    metric: "every Rust binary subtraction whose left operand is a .len() or .count() call without arguments, outside test code, without a DECISION: or INVARIANT: reference comment on the line directly above it",
    skill: "subtract-checked",
    parameters: Parameters::Level {
        default: Severity::Error,
        violation: "subtracting from a length underflows when the collection is shorter, panicking in debug builds and wrapping to a huge index in release; use checked_sub or saturating_sub and handle the short case, or put a DECISION: or INVARIANT: reference on the line directly above it naming the check that guarantees the length",
    },
};
const REPEATEDFRAGMENT: Descriptor = Descriptor {
    default_id: "repeated-fragment",
    exclude: TEST_CODE,
    target: "file",
    metric: "every Rust macro_rules! arm whose transcriber names a $x bound as an expr or expr_2021 fragment by its matcher two or more times outside stringify!(..), reported at the second use, outside test code, without a DECISION: or INVARIANT: reference comment on the line directly above it",
    skill: "bind-the-fragment",
    parameters: Parameters::Level {
        default: Severity::Error,
        violation: "an expression fragment named twice in a macro body is evaluated twice, so an argument with a side effect runs twice and the result silently differs; bind it once with let at the start of the body and use the binding, or put a DECISION: or INVARIANT: reference on the line directly above it",
    },
};
const HIDDENEXIT: Descriptor = Descriptor {
    default_id: "hidden-exit",
    exclude: TEST_CODE,
    target: "file",
    metric: "every return, ? and break or continue outside a loop of the same body in a Rust macro_rules! transcriber that parses as a function body once each $ and the delimiters, separator and operator of each $(..) repetition become spaces, outside a closure, nested fn or async block of that body, outside test code, without a DECISION: or INVARIANT: reference comment on the line directly above it",
    skill: "expose-the-exit",
    parameters: Parameters::Level {
        default: Severity::Error,
        violation: "a macro that returns, propagates with ? or breaks the caller's loop looks like an expression at the call site, so a reader or an agent does not see the exit; turn it into a function returning Result or Option and write the ? at the call site, or put a DECISION: or INVARIANT: reference on the line directly above the exit",
    },
};
