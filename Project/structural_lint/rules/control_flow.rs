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
        }
    }
}
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
    metric: "operands of one Rust && and || or Python and and or expression anywhere in code: parentheses group operands without ending the expression, a let binding of a Rust let chain is not an operand, and a negation, comparison, call, closure or macro is one operand whose inner expressions are measured on their own",
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
