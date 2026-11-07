use super::descriptor::Descriptor;
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
}
impl ControlFlow {
    pub(super) fn descriptor(self) -> &'static Descriptor {
        match self {
            Self::NamedIfCondition => &NAMEDIFCONDITION,
            Self::NestingDepth => &NESTINGDEPTH,
            Self::BranchBudget => &BRANCHBUDGET,
            Self::BooleanExpression => &BOOLEANEXPRESSION,
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
