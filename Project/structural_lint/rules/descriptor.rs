use super::kind::Kind;
use serde_json::{Value, json};

pub struct Descriptor {
    pub default_id: &'static str,
    pub target: &'static str,
    pub metric: &'static str,
    pub skill: &'static str,
    pub parameters: Parameters,
}
pub enum Parameters {
    Thresholds { warning: Option<u64>, error: u64 },
    Level { violation: &'static str },
}
impl Parameters {
    pub fn describe(&self) -> Value {
        match self {
            Self::Thresholds { .. } => {
                json!({"type": "thresholds", "warning": "optional nonnegative integer", "error": "optional nonnegative integer", "constraint": "at least one threshold; warning < error; finding when actual > threshold", "overrides": "ordered; last matching value wins"})
            }
            Self::Level { .. } => {
                json!({"type": "policy", "level": ["warning", "error"], "constraint": "level required; thresholds and overrides forbidden"})
            }
        }
    }
    pub fn apply(&self, value: &mut Value) {
        match self {
            Self::Level { .. } => value["level"] = json!("error"),
            Self::Thresholds { warning, error } => {
                value["error"] = json!(error);
                if let Some(warning) = warning {
                    value["warning"] = json!(warning);
                }
            }
        }
    }
}
impl Kind {
    pub fn descriptor(self) -> &'static Descriptor {
        match self {
            Self::NonblankLines => &NONBLANKLINES,
            Self::LineWidth => &LINEWIDTH,
            Self::ForbiddenFile => &FORBIDDENFILE,
            Self::DirectoryEntries => &DIRECTORYENTRIES,
            Self::NamedIfCondition => &NAMEDIFCONDITION,
            Self::FunctionLines => &FUNCTIONLINES,
            Self::ParameterCount => &PARAMETERCOUNT,
            Self::NestingDepth => &NESTINGDEPTH,
            Self::BooleanParameter => &BOOLEANPARAMETER,
            Self::CommentReference => &COMMENTREFERENCE,
            Self::DirectoryArchitecture => &DIRECTORYARCHITECTURE,
        }
    }
}
const NONBLANKLINES: Descriptor = Descriptor {
    default_id: "file-size",
    target: "file",
    metric: "nonempty lines, including comments; non-UTF-8 files skipped",
    skill: "refactor-large-file",
    parameters: Parameters::Thresholds {
        warning: Some(300),
        error: 500,
    },
};
// DECISION: D036
const LINEWIDTH: Descriptor = Descriptor {
    default_id: "line-width",
    target: "file",
    metric: "characters per line, one finding per line over a threshold; non-UTF-8 files skipped",
    skill: "condense-markdown",
    parameters: Parameters::Thresholds {
        warning: None,
        error: 120,
    },
};
const FORBIDDENFILE: Descriptor = Descriptor {
    default_id: "forbidden-file",
    target: "file",
    metric: "every selected file is a finding, regardless of content",
    skill: "condense-markdown",
    parameters: Parameters::Level {
        violation: "file is not allowed here; move needed content to its owner and delete the file",
    },
};
// DECISION: D037
const COMMENTREFERENCE: Descriptor = Descriptor {
    default_id: "comments",
    target: "file",
    metric: "every comment and lone string statement except an exact DECISION: DNNN or INVARIANT: INNN reference",
    skill: "replace-comment",
    parameters: Parameters::Level {
        violation: "only an exact DECISION: DNNN or INVARIANT: INNN comment is allowed; move the explanation to its owner or delete it",
    },
};
const DIRECTORYARCHITECTURE: Descriptor = Descriptor {
    default_id: "architecture",
    target: "directory",
    metric: "architecture.yaml inventories and directory cycles in resolved source dependencies; incomplete analysis is a finding",
    skill: "refactor-large-directory",
    parameters: Parameters::Level {
        violation: "architecture contract is violated",
    },
};

const DIRECTORYENTRIES: Descriptor = Descriptor {
    default_id: "directory-size",
    target: "directory",
    metric: "immediate child names in the selected inventory",
    skill: "refactor-large-directory",
    parameters: Parameters::Thresholds {
        warning: Some(10),
        error: 15,
    },
};

const NAMEDIFCONDITION: Descriptor = Descriptor {
    default_id: "named-if",
    target: "file",
    metric: "one named value per boolean if condition; Rust if-let bindings excluded; named field paths accepted",
    skill: "name-if-condition",
    parameters: Parameters::Level {
        violation: "if condition must be one named value; give the branch reason a name",
    },
};

const FUNCTIONLINES: Descriptor = Descriptor {
    default_id: "function-size",
    target: "file",
    metric: "nonblank lines from signature through body, including comments; declarations without body excluded",
    skill: "refactor-long-function",
    parameters: Parameters::Thresholds {
        warning: None,
        error: 40,
    },
};

const PARAMETERCOUNT: Descriptor = Descriptor {
    default_id: "parameters",
    target: "file",
    metric: "declared input parameters, excluding method receivers; constructors included; variadics count as one",
    skill: "reduce-parameters",
    parameters: Parameters::Thresholds {
        warning: None,
        error: 4,
    },
};

// DECISION: D061
const NESTINGDEPTH: Descriptor = Descriptor {
    default_id: "nesting-depth",
    target: "file",
    metric: "deepest nested control-flow blocks in a function; if/match/loop/while/for add depth; Rust else-if and Python elif continue the same level; nested function and closure bodies are measured on their own",
    skill: "flatten-nesting",
    parameters: Parameters::Thresholds {
        warning: None,
        error: 4,
    },
};

// DECISION: D062
const BOOLEANPARAMETER: Descriptor = Descriptor {
    default_id: "boolean-parameter",
    target: "file",
    metric: "parameters typed as the Rust bool primitive, or annotated bool or defaulting to True or False in Python; each flagged parameter is a finding",
    skill: "split-boolean-parameter",
    parameters: Parameters::Level {
        violation: "a boolean parameter hides two behaviors; split the function or name the modes with a dedicated type",
    },
};
