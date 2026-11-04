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
            Self::DumpName => &DUMPNAME,
            Self::NamedIfCondition => &NAMEDIFCONDITION,
            Self::FunctionLines => &FUNCTIONLINES,
            Self::ParameterCount => &PARAMETERCOUNT,
            Self::NestingDepth => &NESTINGDEPTH,
            Self::BooleanParameter => &BOOLEANPARAMETER,
            Self::BranchBudget => &BRANCHBUDGET,
            Self::UnsafeJustification => &UNSAFEJUSTIFICATION,
            Self::HardcodedSecret => &HARDCODEDSECRET,
            Self::SilentError => &SILENTERROR,
            Self::DisabledTest => &DISABLEDTEST,
            Self::AllowJustification => &ALLOWJUSTIFICATION,
            Self::DebugArtifact => &DEBUGARTIFACT,
            Self::DebugAssertion => &DEBUGASSERTION,
            Self::PanicPath => &PANICPATH,
            Self::WildcardImport => &WILDCARDIMPORT,
            Self::TupleAsRecord => &TUPLEASRECORD,
            Self::EagerFallback => &EAGERFALLBACK,
            Self::DiscardedResult => &DISCARDEDRESULT,
            Self::DisabledRaii => &DISABLEDRAII,
            Self::GlobalState => &GLOBALSTATE,
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

// DECISION: D072
const DUMPNAME: Descriptor = Descriptor {
    default_id: "dump-name",
    target: "directory",
    metric: "a directory whose name is a generic dumping ground: utils, helpers, misc or common; shared is allowed",
    skill: "refactor-large-directory",
    parameters: Parameters::Level {
        violation: "a directory named for a generic dumping ground hides mixed responsibilities; name it for what it owns or distribute its contents to cohesive owners; shared is for genuine cross-feature leaves",
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
// DECISION: D063
const BRANCHBUDGET: Descriptor = Descriptor {
    default_id: "branch-budget",
    target: "file",
    metric: "decision points in a function: each if, else-if and ternary condition, and each match or case as one multi-way decision regardless of arms; nested function and closure bodies are measured on their own",
    skill: "reduce-branching",
    parameters: Parameters::Thresholds {
        warning: None,
        error: 5,
    },
};
// DECISION: D067
const HARDCODEDSECRET: Descriptor = Descriptor {
    default_id: "hardcoded-secret",
    target: "file",
    metric: "every string literal matching a known credential format: a private-key block, or an AWS, GitHub, Google or Slack token",
    skill: "remove-secret",
    parameters: Parameters::Level {
        violation: "a hardcoded credential does not belong in source; move it to configuration or a secret store and rotate the exposed value",
    },
};
// DECISION: D069
// DECISION: D097
const SILENTERROR: Descriptor = Descriptor {
    default_id: "silent-error",
    target: "file",
    metric: "every Python except clause whose body is only pass, every Rust read_dir call chain collapsed by .flatten(), .filter_map(Result::ok), .filter_map(|e| e.ok()) or .map_while(Result::ok), and every Rust match or if-let arm Err(_)/Err(..) whose body is Ok(true), Ok(false) or a return of one inside a function declared to return Result<bool, ...>, outside Rust test code, without a DECISION: or INVARIANT: reference comment on the line directly above the pass, the chain or the arm",
    skill: "handle-error",
    parameters: Parameters::Level {
        violation: "an except that only passes, a read_dir chain that drops its failed entries or an Err arm that answers a verdict swallows the error; handle it, propagate or re-raise it, or put a DECISION: or INVARIANT: reference on the line directly above to justify ignoring it",
    },
};
// DECISION: D074
const DISABLEDTEST: Descriptor = Descriptor {
    default_id: "disabled-test",
    target: "file",
    metric: "every Rust #[ignore] attribute or Python @pytest.mark.skip or skipif decorator without a DECISION: or INVARIANT: reference comment on the line directly above it",
    skill: "justify-skip",
    parameters: Parameters::Level {
        violation: "a silently disabled test drops coverage; re-enable it, delete it if obsolete, or put a DECISION: or INVARIANT: reference on the line directly above it to justify skipping",
    },
};
// DECISION: D080
const ALLOWJUSTIFICATION: Descriptor = Descriptor {
    default_id: "allow-justification",
    target: "file",
    metric: "every Rust #[allow(...)] or #![allow(...)] attribute without a DECISION: or INVARIANT: reference comment on the line directly above it",
    skill: "justify-allow",
    parameters: Parameters::Level {
        violation: "a blanket allow silences a rule; remove it and fix the underlying finding, or put a DECISION: or INVARIANT: reference on the line directly above it to justify the suppression",
    },
};
// DECISION: D081
const DEBUGARTIFACT: Descriptor = Descriptor {
    default_id: "debug-artifact",
    target: "file",
    metric: "every Rust dbg! macro invocation",
    skill: "remove-debug",
    parameters: Parameters::Level {
        violation: "a dbg! macro is a leftover debugging artifact; remove it and keep any value it wrapped",
    },
};
// DECISION: D084
const DEBUGASSERTION: Descriptor = Descriptor {
    default_id: "debug-assertion",
    target: "file",
    metric: "every Rust debug_assert!, debug_assert_eq! or debug_assert_ne! macro outside test code without a DECISION: or INVARIANT: reference comment on the line directly above it",
    skill: "firm-up-assertion",
    parameters: Parameters::Level {
        violation: "a debug_assert is stripped from release builds, so the contract it states goes unchecked where it matters; promote it to assert! to hold in every build, remove it if redundant, or put a DECISION: or INVARIANT: reference on the line directly above it to justify the debug-only check",
    },
};
// DECISION: D082
const PANICPATH: Descriptor = Descriptor {
    default_id: "panic-path",
    target: "file",
    metric: "every Rust .unwrap() or .expect(...) call and panic!, todo!, unimplemented! or unreachable! macro outside test code without a DECISION: or INVARIANT: reference comment on the line directly above it",
    skill: "handle-panic",
    parameters: Parameters::Level {
        violation: "an unwrap, expect or panic macro aborts the program on failure; return or handle the error with ?, Result or match, or put a DECISION: or INVARIANT: reference on the line directly above it to justify the panic",
    },
};
// DECISION: D085
const WILDCARDIMPORT: Descriptor = Descriptor {
    default_id: "wildcard-import",
    target: "file",
    metric: "every Rust use glob (`use path::*`) outside test code, or Python wildcard import (`from module import *`), without a DECISION: or INVARIANT: reference comment on the line directly above it",
    skill: "narrow-import",
    parameters: Parameters::Level {
        violation: "a glob import binds the whole, evolving surface of another module, so a name added or renamed upstream silently changes what is in scope; replace it with an explicit import list, or put a DECISION: or INVARIANT: reference on the line directly above it to justify the glob",
    },
};
// DECISION: D065
const UNSAFEJUSTIFICATION: Descriptor = Descriptor {
    default_id: "unsafe-justification",
    target: "file",
    metric: "every Rust unsafe block or unsafe function without a DECISION: or INVARIANT: reference comment on the line directly above it",
    skill: "justify-unsafe",
    parameters: Parameters::Level {
        violation: "unsafe needs a recorded justification; put a DECISION: or INVARIANT: reference on the line directly above it, or remove the unsafe",
    },
};
// DECISION: D088
const TUPLEASRECORD: Descriptor = Descriptor {
    default_id: "tuple-as-record",
    target: "file",
    metric: "every Rust function parameter type or return type that is a tuple, outside test code, without a DECISION: or INVARIANT: reference comment on the line directly above the function",
    skill: "name-the-record",
    parameters: Parameters::Level {
        violation: "a tuple in a signature is an unnamed record that evades the parameter-count and boolean-parameter budgets and hides what each position means; give it a named struct, or put a DECISION: or INVARIANT: reference on the line directly above the function to justify the tuple",
    },
};
// DECISION: D089
const EAGERFALLBACK: Descriptor = Descriptor {
    default_id: "eager-fallback",
    target: "file",
    metric: "every Rust .unwrap_or, .ok_or or .map_or call whose eager first argument propagates with ? outside test code, without a DECISION: or INVARIANT: reference comment on the line directly above it",
    skill: "eager-to-lazy",
    parameters: Parameters::Level {
        violation: "the fallback argument of unwrap_or, ok_or or map_or is evaluated unconditionally, so a ? inside it propagates even when the value is present; move the fallback into the lazy unwrap_or_else, ok_or_else or map_or_else closure, or put a DECISION: or INVARIANT: reference on the line directly above it to justify the eager form",
    },
};
// DECISION: D090
const DISCARDEDRESULT: Descriptor = Descriptor {
    default_id: "discarded-result",
    target: "file",
    metric: "every Rust bare .ok() statement that drops a Result outside test code, without a DECISION: or INVARIANT: reference comment on the line directly above it",
    skill: "handle-discarded",
    parameters: Parameters::Level {
        violation: "a bare .ok() statement converts a Result to Option and throws the outcome away, so a failure passes silently; handle the error, propagate it with ?, or put a DECISION: or INVARIANT: reference on the line directly above it to justify discarding it",
    },
};
// DECISION: D092
const DISABLEDRAII: Descriptor = Descriptor {
    default_id: "disabled-raii",
    target: "file",
    metric: "every Rust .keep() or .into_path() method call, or mem::forget, ManuallyDrop::new or Box::leak path call, outside test code without a DECISION: or INVARIANT: reference comment on the line directly above it",
    skill: "own-the-resource",
    parameters: Parameters::Level {
        violation: "this call defeats RAII, so a guard's cleanup never runs and a resource outlives its owner silently; let the value drop, hold the guard for the lifetime it protects, or put a DECISION: or INVARIANT: reference on the line directly above it to justify keeping, forgetting or leaking it",
    },
};
// DECISION: D096
const GLOBALSTATE: Descriptor = Descriptor {
    default_id: "global-state",
    target: "file",
    metric: "every Rust static item outside test code, excluding static mut, whose type (following same-file type aliases up to three levels) contains a lock (Mutex, RwLock, DashMap, DashSet), or a public static whose type contains any cell (also RefCell, Cell, UnsafeCell, Atomic*), including items declared inside lazy_static! or thread_local!, without a DECISION: or INVARIANT: reference comment on the line directly above it",
    skill: "own-the-state",
    parameters: Parameters::Level {
        violation: "a static lock or a public static cell is process-global mutable state with no owner in the directory tree, so every writer has to be found by search; give the state an owner (a parameter, a context field or a private cell behind one owning function), or put a DECISION: or INVARIANT: reference on the line directly above it to justify the global",
    },
};
