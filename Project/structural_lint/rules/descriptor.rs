use super::kind::Kind;
use serde_json::{Value, json};

pub struct Descriptor {
    pub default_id: &'static str,
    pub exclude: &'static [&'static str],
    pub target: &'static str,
    pub metric: &'static str,
    pub skill: &'static str,
    pub parameters: Parameters,
}
const TEST_CODE: &[&str] = &["**/tests/**", "**/tests.rs"];
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
            Self::SuperSuperReach => &SUPERSUPERREACH,
            Self::RecursiveDeletion => &RECURSIVEDELETION,
            Self::UnwindJustification => &UNWINDJUSTIFICATION,
            Self::CommentReference => &COMMENTREFERENCE,
            Self::DirectoryArchitecture => &DIRECTORYARCHITECTURE,
        }
    }
}
const NONBLANKLINES: Descriptor = Descriptor {
    default_id: "file-size",
    exclude: &[],
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
    exclude: &[],
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
    exclude: &[],
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
    exclude: &[],
    target: "file",
    metric: "every comment and lone string statement except an exact DECISION: DNNN or INVARIANT: INNN reference",
    skill: "replace-comment",
    parameters: Parameters::Level {
        violation: "only an exact DECISION: DNNN or INVARIANT: INNN comment is allowed; move the explanation to its owner or delete it",
    },
};
const DIRECTORYARCHITECTURE: Descriptor = Descriptor {
    default_id: "architecture",
    exclude: &[],
    target: "directory",
    metric: "architecture.yaml inventories and directory cycles in resolved source dependencies; incomplete analysis is a finding",
    skill: "refactor-large-directory",
    parameters: Parameters::Level {
        violation: "architecture contract is violated",
    },
};

const DIRECTORYENTRIES: Descriptor = Descriptor {
    default_id: "directory-size",
    exclude: &[],
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
    exclude: &[],
    target: "directory",
    metric: "a directory whose name is a generic dumping ground: utils, helpers, misc or common; shared is allowed",
    skill: "refactor-large-directory",
    parameters: Parameters::Level {
        violation: "a directory named for a generic dumping ground hides mixed responsibilities; name it for what it owns or distribute its contents to cohesive owners; shared is for genuine cross-feature leaves",
    },
};

const NAMEDIFCONDITION: Descriptor = Descriptor {
    default_id: "named-if",
    exclude: &[],
    target: "file",
    metric: "one named value per boolean if condition; Rust if-let bindings excluded; named field paths accepted",
    skill: "name-if-condition",
    parameters: Parameters::Level {
        violation: "if condition must be one named value; give the branch reason a name",
    },
};

const FUNCTIONLINES: Descriptor = Descriptor {
    default_id: "function-size",
    exclude: &[],
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
    exclude: &[],
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
    exclude: &[],
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
    exclude: &[],
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
    exclude: &[],
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
    exclude: &[],
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
    exclude: &[],
    target: "file",
    metric: "every Python except clause whose body is only pass, every Rust read_dir call chain collapsed by .flatten(), .filter_map(Result::ok), .filter_map(|e| e.ok()) or .map_while(Result::ok), and every Rust match or if-let arm Err(_)/Err(..) whose body is Ok(true), Ok(false) or a return of one inside a function declared to return Result<bool, ...>, outside Rust test code, without a DECISION: or INVARIANT: reference comment on the line directly above the pass, the chain or the arm",
    skill: "handle-error",
    parameters: Parameters::Level {
        violation: "an except that only passes, a read_dir chain that drops its failed entries or an Err arm that answers a verdict swallows the error; handle it, propagate or re-raise it, or put a DECISION: or INVARIANT: reference on the line directly above to justify ignoring it",
    },
};
// DECISION: D074
// DECISION: D098
const DISABLEDTEST: Descriptor = Descriptor {
    default_id: "disabled-test",
    exclude: &[],
    target: "file",
    metric: "every Rust #[ignore] attribute, also as a direct #[cfg_attr(...)] argument, or Python @pytest.mark.skip or skipif decorator without a DECISION: or INVARIANT: reference comment on the line directly above it",
    skill: "justify-skip",
    parameters: Parameters::Level {
        violation: "a silently disabled test drops coverage; re-enable it, delete it if obsolete, or put a DECISION: or INVARIANT: reference on the line directly above it to justify skipping",
    },
};
// DECISION: D080
// DECISION: D098
const ALLOWJUSTIFICATION: Descriptor = Descriptor {
    default_id: "allow-justification",
    exclude: &[],
    target: "file",
    metric: "every Rust #[allow(...)], #![allow(...)], #[expect(...)] or #![expect(...)] attribute, every #[cfg_attr(...)] carrying allow, expect or ignore as a direct argument, and every lint set to \"allow\" or { level = \"allow\" } under a [lints.*] or [workspace.lints.*] table of a TOML manifest, without a DECISION: or INVARIANT: reference comment on the line directly above the attribute or the key",
    skill: "justify-allow",
    parameters: Parameters::Level {
        violation: "a blanket allow, expect or conditional suppression silences a rule, and a reason = \"...\" inside the attribute is not a reference; remove it and fix the underlying finding, or put a DECISION: or INVARIANT: reference on the line directly above it to justify the suppression",
    },
};
// DECISION: D081
const DEBUGARTIFACT: Descriptor = Descriptor {
    default_id: "debug-artifact",
    exclude: &[],
    target: "file",
    metric: "every Rust dbg! macro invocation",
    skill: "remove-debug",
    parameters: Parameters::Level {
        violation: "a dbg! macro is a leftover debugging artifact; remove it and keep any value it wrapped",
    },
};
// DECISION: D084
// DECISION: D115
const DEBUGASSERTION: Descriptor = Descriptor {
    default_id: "debug-assertion",
    exclude: TEST_CODE,
    target: "file",
    metric: "every Rust debug_assert!, debug_assert_eq! or debug_assert_ne! macro, and every assert!, assert_eq! or assert_ne! macro under a cfg(debug_assertions) attribute or in the then-branch of if cfg!(debug_assertions), outside test code without a DECISION: or INVARIANT: reference comment on the line directly above it",
    skill: "firm-up-assertion",
    parameters: Parameters::Level {
        violation: "a debug_assert, or an assert compiled only with debug_assertions, is stripped from release builds, so the contract it states goes unchecked where it matters; promote it to an unconditional assert! to hold in every build, remove it if redundant, or put a DECISION: or INVARIANT: reference on the line directly above it to justify the debug-only check",
    },
};
// DECISION: D082
// DECISION: D115
const PANICPATH: Descriptor = Descriptor {
    default_id: "panic-path",
    exclude: TEST_CODE,
    target: "file",
    metric: "every Rust .unwrap(), .expect(...), .unwrap_err() or .expect_err(...) call, resume_unwind call by path or bare name, and panic!, todo!, unimplemented! or unreachable! macro outside test code without a DECISION: or INVARIANT: reference comment on the line directly above it",
    skill: "handle-panic",
    parameters: Parameters::Level {
        violation: "an unwrap, expect, unwrap_err, expect_err, resume_unwind or panic macro aborts the program on failure; return or handle the error with ?, Result or match, or put a DECISION: or INVARIANT: reference on the line directly above it to justify the panic",
    },
};
// DECISION: D085
const WILDCARDIMPORT: Descriptor = Descriptor {
    default_id: "wildcard-import",
    exclude: TEST_CODE,
    target: "file",
    metric: "every Rust use glob (`use path::*`) outside test code, or Python wildcard import (`from module import *`), without a DECISION: or INVARIANT: reference comment on the line directly above it",
    skill: "narrow-import",
    parameters: Parameters::Level {
        violation: "a glob import binds the whole, evolving surface of another module, so a name added or renamed upstream silently changes what is in scope; replace it with an explicit import list, or put a DECISION: or INVARIANT: reference on the line directly above it to justify the glob",
    },
};
// DECISION: D065
// DECISION: D103
const UNSAFEJUSTIFICATION: Descriptor = Descriptor {
    default_id: "unsafe-justification",
    exclude: &[],
    target: "file",
    metric: "every Rust unsafe site (an unsafe block, unsafe function, unsafe impl, unsafe trait or static mut item, and every unsafe keyword inside a macro_rules! body or a macro invocation's arguments except one opening a #[unsafe(..)] or #[cfg_attr(cond, unsafe(..))] attribute) without a DECISION: or INVARIANT: reference comment on the line directly above it, where attribute lines between the comment and an item keep the adjacency",
    skill: "justify-unsafe",
    parameters: Parameters::Level {
        violation: "unsafe needs a recorded justification; put a DECISION: or INVARIANT: reference on the line directly above it (above its attributes for an item), or remove the unsafe",
    },
};
// DECISION: D088
// DECISION: D101
const TUPLEASRECORD: Descriptor = Descriptor {
    default_id: "tuple-as-record",
    exclude: TEST_CODE,
    target: "file",
    metric: "every Rust function parameter type or return type that is a tuple, or that nests inside a generic or slice type, at any depth, a tuple of three or more elements with a primitive, String or &str element (closure signatures included for the nested form), outside test code, without a DECISION: or INVARIANT: reference comment on the line directly above the function",
    skill: "name-the-record",
    parameters: Parameters::Level {
        violation: "a tuple in a signature is an unnamed record that evades the parameter-count and boolean-parameter budgets and hides what each position means, whether it stands alone or inside Vec<(..)>, Option<(..)> or &[(..)]; give it a named struct, or put a DECISION: or INVARIANT: reference on the line directly above the function to justify the tuple",
    },
};
// DECISION: D089
const EAGERFALLBACK: Descriptor = Descriptor {
    default_id: "eager-fallback",
    exclude: TEST_CODE,
    target: "file",
    metric: "every Rust .unwrap_or, .ok_or or .map_or call whose eager first argument propagates with ? outside test code, without a DECISION: or INVARIANT: reference comment on the line directly above it",
    skill: "eager-to-lazy",
    parameters: Parameters::Level {
        violation: "the fallback argument of unwrap_or, ok_or or map_or is evaluated unconditionally, so a ? inside it propagates even when the value is present; move the fallback into the lazy unwrap_or_else, ok_or_else or map_or_else closure, or put a DECISION: or INVARIANT: reference on the line directly above it to justify the eager form",
    },
};
// DECISION: D090
// DECISION: D099
// DECISION: D114
const DISCARDEDRESULT: Descriptor = Descriptor {
    default_id: "discarded-result",
    exclude: TEST_CODE,
    target: "file",
    metric: "every Rust bare .ok() or .err() statement that drops a Result, every bare statement whose last call is .map, .and_then, .map_or or .map_or_else and drops the combined value, and every bare .status()? or .output()? statement that propagates only the launch error and drops the exit status or output of a command, outside test code, without a DECISION: or INVARIANT: reference comment on the line directly above it",
    skill: "handle-discarded",
    parameters: Parameters::Level {
        violation: "a bare .ok() or .err() statement converts a Result to Option and throws the outcome away, a bare .map, .and_then, .map_or or .map_or_else statement throws away the value the closure produced, and a bare .status()? or .output()? statement keeps only the launch error and throws the exit status or output away, so a failure passes silently; handle the error or the value, propagate it, or put a DECISION: or INVARIANT: reference on the line directly above it to justify discarding it",
    },
};
// DECISION: D092
const DISABLEDRAII: Descriptor = Descriptor {
    default_id: "disabled-raii",
    exclude: TEST_CODE,
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
    exclude: TEST_CODE,
    target: "file",
    metric: "every Rust static item outside test code, excluding static mut, whose type (following same-file type aliases up to three levels) contains a lock (Mutex, RwLock, DashMap, DashSet), or a public static whose type contains any cell (also RefCell, Cell, UnsafeCell, Atomic*), including items declared inside lazy_static! or thread_local!, without a DECISION: or INVARIANT: reference comment on the line directly above it",
    skill: "own-the-state",
    parameters: Parameters::Level {
        violation: "a static lock or a public static cell is process-global mutable state with no owner in the directory tree, so every writer has to be found by search; give the state an owner (a parameter, a context field or a private cell behind one owning function), or put a DECISION: or INVARIANT: reference on the line directly above it to justify the global",
    },
};
// DECISION: D100
const SUPERSUPERREACH: Descriptor = Descriptor {
    default_id: "super-super-reach",
    exclude: TEST_CODE,
    target: "file",
    metric: "every Rust path that reaches two module levels up through super::super, in a use declaration, an expression or a type, outside test code, without a DECISION: or INVARIANT: reference comment on the line directly above it",
    skill: "anchor-at-crate",
    parameters: Parameters::Level {
        violation: "a super::super path binds the file to its position in the module tree instead of to the item's owner, so moving a directory breaks it and a reader cannot see whose item it is; write the path from crate::, or re-export the item at the parent module and reach it through a single super::, or put a DECISION: or INVARIANT: reference on the line directly above it to justify the reach",
    },
};
// DECISION: D102
const RECURSIVEDELETION: Descriptor = Descriptor {
    default_id: "recursive-deletion",
    exclude: &[],
    target: "file",
    metric: "every Rust call whose function path ends in remove_dir_all, whose bare function name is remove_dir_all or whose method is remove_dir_all, from any source such as std::fs, tokio::fs, a local wrapper or a method, in source and in macro bodies, outside test code, without a DECISION: or INVARIANT: reference comment on the line directly above it",
    skill: "contain-the-deletion",
    parameters: Parameters::Level {
        violation: "a recursive directory deletion is the most destructive file-system operation, and a wrong argument erases a working tree; delete through an owner that checks the path lies inside its own area, or put a DECISION: or INVARIANT: reference on the line directly above the call to justify the deletion",
    },
};
// DECISION: D115
const UNWINDJUSTIFICATION: Descriptor = Descriptor {
    default_id: "unwind-justification",
    exclude: TEST_CODE,
    target: "file",
    metric: "every Rust call whose function path or bare name ends in catch_unwind or AssertUnwindSafe, or whose method is catch_unwind, outside test code, without a DECISION: or INVARIANT: reference comment on the line directly above it",
    skill: "justify-unwind",
    parameters: Parameters::Level {
        violation: "catching a panic turns a crash into a value and AssertUnwindSafe vouches that state touched by the panic is still consistent, so a bug can continue silently on broken state; let the panic propagate and return errors with Result, or catch only at a boundary that must not unwind, such as FFI or task isolation, and put a DECISION: or INVARIANT: reference on the line directly above the call to justify it",
    },
};
