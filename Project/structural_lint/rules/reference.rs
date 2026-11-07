use super::descriptor::{Descriptor, TEST_CODE};
use super::parameters::Parameters;
use crate::shared::diagnostics::lint::Level as Severity;
use serde::{Deserialize, Serialize};

// DECISION: D133
#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum Reference {
    UnsafeJustification,
    HardcodedSecret,
    SilentError,
    DisabledTest,
    AllowJustification,
    DebugArtifact,
    DebugAssertion,
    PanicPath,
    WildcardImport,
    EagerFallback,
    DiscardedResult,
    SuperSuperReach,
    UnwindJustification,
    BlockingInAsync,
    CommentReference,
}
impl Reference {
    pub(super) fn descriptor(self) -> &'static Descriptor {
        match self {
            Self::UnsafeJustification => &UNSAFEJUSTIFICATION,
            Self::HardcodedSecret => &HARDCODEDSECRET,
            Self::SilentError => &SILENTERROR,
            Self::DisabledTest => &DISABLEDTEST,
            Self::AllowJustification => &ALLOWJUSTIFICATION,
            Self::DebugArtifact => &DEBUGARTIFACT,
            Self::DebugAssertion => &DEBUGASSERTION,
            Self::PanicPath => &PANICPATH,
            Self::WildcardImport => &WILDCARDIMPORT,
            Self::EagerFallback => &EAGERFALLBACK,
            Self::DiscardedResult => &DISCARDEDRESULT,
            Self::SuperSuperReach => &SUPERSUPERREACH,
            Self::UnwindJustification => &UNWINDJUSTIFICATION,
            Self::BlockingInAsync => &BLOCKINGINASYNC,
            Self::CommentReference => &COMMENTREFERENCE,
        }
    }
}
// DECISION: D065
// DECISION: D103
const UNSAFEJUSTIFICATION: Descriptor = Descriptor {
    default_id: "unsafe-justification",
    exclude: &[],
    target: "file",
    metric: "every Rust unsafe site (an unsafe block, unsafe function, unsafe impl, unsafe trait or static mut item, and every unsafe keyword inside a macro_rules! body or a macro invocation's arguments except one opening a #[unsafe(..)] or #[cfg_attr(cond, unsafe(..))] attribute) without a DECISION: or INVARIANT: reference comment on the line directly above it, where attribute lines between the comment and an item keep the adjacency",
    skill: "justify-unsafe",
    parameters: Parameters::Level {
        default: Severity::Error,
        violation: "unsafe needs a recorded justification; put a DECISION: or INVARIANT: reference on the line directly above it (above its attributes for an item), or remove the unsafe",
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
        default: Severity::Error,
        violation: "a hardcoded credential does not belong in source; move it to configuration or a secret store and rotate the exposed value",
    },
};
// DECISION: D069
// DECISION: D097
// DECISION: D126
const SILENTERROR: Descriptor = Descriptor {
    default_id: "silent-error",
    exclude: &[],
    target: "file",
    metric: "every Python except clause whose body is only pass, every Rust read_dir call chain collapsed by .flatten(), .filter_map(Result::ok), .filter_map(|e| e.ok()) or .map_while(Result::ok), every Rust match or if-let arm Err(_)/Err(..) whose body is Ok(true), Ok(false) or a return of one inside a function declared to return Result<bool, ...>, and every Ok(<status>.success()) returned as the tail or by return from such a function whose status comes from .status()?, .output()?.status or a let binding of them while its body reads no .code(), outside Rust test code, without a DECISION: or INVARIANT: reference comment on the line directly above the pass, the chain, the arm or the return",
    skill: "handle-error",
    parameters: Parameters::Level {
        default: Severity::Error,
        violation: "an except that only passes, a read_dir chain that drops its failed entries, an Err arm that answers a verdict or a returned .success() of a process status that never reads .code() swallows the error; handle it, propagate or re-raise it, compare code() with the codes the command documents as answers and refuse the others or call an owner that does, or put a DECISION: or INVARIANT: reference on the line directly above to justify ignoring it",
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
        default: Severity::Error,
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
        default: Severity::Error,
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
        default: Severity::Error,
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
        default: Severity::Error,
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
        default: Severity::Error,
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
        default: Severity::Error,
        violation: "a glob import binds the whole, evolving surface of another module, so a name added or renamed upstream silently changes what is in scope; replace it with an explicit import list, or put a DECISION: or INVARIANT: reference on the line directly above it to justify the glob",
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
        default: Severity::Error,
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
        default: Severity::Error,
        violation: "a bare .ok() or .err() statement converts a Result to Option and throws the outcome away, a bare .map, .and_then, .map_or or .map_or_else statement throws away the value the closure produced, and a bare .status()? or .output()? statement keeps only the launch error and throws the exit status or output away, so a failure passes silently; handle the error or the value, propagate it, or put a DECISION: or INVARIANT: reference on the line directly above it to justify discarding it",
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
        default: Severity::Error,
        violation: "a super::super path binds the file to its position in the module tree instead of to the item's owner, so moving a directory breaks it and a reader cannot see whose item it is; write the path from crate::, or re-export the item at the parent module and reach it through a single super::, or put a DECISION: or INVARIANT: reference on the line directly above it to justify the reach",
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
        default: Severity::Error,
        violation: "catching a panic turns a crash into a value and AssertUnwindSafe vouches that state touched by the panic is still consistent, so a bug can continue silently on broken state; let the panic propagate and return errors with Result, or catch only at a boundary that must not unwind, such as FFI or task isolation, and put a DECISION: or INVARIANT: reference on the line directly above the call to justify it",
    },
};
// DECISION: D118
const BLOCKINGINASYNC: Descriptor = Descriptor {
    default_id: "blocking-in-async",
    exclude: TEST_CODE,
    target: "file",
    metric: "every Rust call whose path ends in thread::sleep, a bare sleep imported from a thread module in the same file, or a function or method named block_on, inside an async fn or async block with no closure or ordinary fn between the call and that boundary, outside test code, without a DECISION: or INVARIANT: reference comment on the line directly above the call",
    skill: "await-not-block",
    parameters: Parameters::Level {
        default: Severity::Warning,
        violation: "a blocking thread::sleep or nested block_on inside async code stalls the executor thread, freezes every task on a single-threaded runtime and panics inside a tokio runtime; await an async sleep or the future itself, move blocking work into spawn_blocking, or put a DECISION: or INVARIANT: reference on the line directly above the call to justify it",
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
        default: Severity::Error,
        violation: "only an exact DECISION: DNNN or INVARIANT: INNN comment is allowed; move the explanation to its owner or delete it",
    },
};
