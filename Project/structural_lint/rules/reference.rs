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
    PositionalFormat,
    CrossedField,
    UntargetedPanic,
    TestSubstitution,
    InterpolatedShell,
    InterpolatedSyntax,
    StaleCheck,
    #[serde(rename = "comment-reference")]
    Comment,
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
            Self::PositionalFormat => &POSITIONALFORMAT,
            Self::CrossedField => &CROSSEDFIELD,
            Self::UntargetedPanic => &UNTARGETEDPANIC,
            Self::TestSubstitution => &TESTSUBSTITUTION,
            Self::InterpolatedShell => &INTERPOLATEDSHELL,
            Self::InterpolatedSyntax => &INTERPOLATEDSYNTAX,
            Self::StaleCheck => &STALECHECK,
            Self::Comment => &COMMENTREFERENCE,
        }
    }
}
// DECISION: D156
const POSITIONALFORMAT: Descriptor = Descriptor {
    default_id: "positional-format",
    exclude: TEST_CODE,
    target: "file",
    metric: "every Rust format, print, println, eprint, eprintln, write, writeln, panic or format_args macro whose first string literal has two or more placeholders without a name, outside test code, without a DECISION: or INVARIANT: reference comment on the line directly above it",
    skill: "name-the-placeholders",
    parameters: Parameters::Level {
        default: Severity::Error,
        violation: "two positional placeholders pair with their arguments only by count, so a swap prints the wrong value; name each placeholder and bind expressions to names first, or put a DECISION: or INVARIANT: reference on the line directly above it",
    },
};
// DECISION: D156
const CROSSEDFIELD: Descriptor = Descriptor {
    default_id: "crossed-field",
    exclude: TEST_CODE,
    target: "file",
    metric: "every Rust struct literal field initialized by a plain identifier that is not its own name but names another field of the same literal, outside test code, without a DECISION: or INVARIANT: reference comment on the line directly above it",
    skill: "align-field-names",
    parameters: Parameters::Level {
        default: Severity::Error,
        violation: "a field set from a variable named after a sibling field is a swap in shorthand clothing; rename the variables so each field takes its own name, or put a DECISION: or INVARIANT: reference on the line directly above it",
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
// DECISION: D165
const SILENTERROR: Descriptor = Descriptor {
    default_id: "silent-error",
    exclude: &[],
    target: "file",
    metric: "every Python except clause whose body is only pass, every Rust read_dir call chain collapsed by .flatten() or .map_while(ok), every Rust .filter_map(ok) or .flat_map(ok) on any chain, where ok is Result::ok or a one-parameter closure whose body is only a .ok() call, except Result::ok and |x| x.ok() on a .lines() call, which Clippy's default lines_filter_map_ok reports, every Rust match or if-let arm Err(_)/Err(..) whose body is Ok(true), Ok(false) or a return of one inside a function declared to return Result<bool, ...>, and every Ok(<status>.success()) returned as the tail or by return from such a function whose status comes from .status()?, .output()?.status or a let binding of them while its body reads no .code(), outside Rust test code, without a DECISION: or INVARIANT: reference comment on the line directly above the pass, the chain, the arm or the return",
    skill: "handle-error",
    parameters: Parameters::Level {
        default: Severity::Error,
        violation: "an except that only passes, a read_dir chain that drops its failed entries, a filter_map or flat_map that keeps only the Ok values, an Err arm that answers a verdict or a returned .success() of a process status that never reads .code() swallows the error; handle it, propagate or re-raise it, collect into a Result or match each Err with the reason for skipping it, compare code() with the codes the command documents as answers and refuse the others or call an owner that does, or put a DECISION: or INVARIANT: reference on the line directly above to justify ignoring it",
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
// DECISION: D147
// DECISION: D148
const PANICPATH: Descriptor = Descriptor {
    default_id: "panic-path",
    exclude: TEST_CODE,
    target: "file",
    metric: "every Rust .unwrap(), .expect(...), .unwrap_err() or .expect_err(...) call, resume_unwind call by path or bare name, and panic!, todo!, unimplemented! or unreachable! macro, also inside a macro_rules! body or a macro invocation's arguments, outside test code without a DECISION: or INVARIANT: reference comment on the line directly above it or above an #[allow] or #[expect] of its Clippy lint on an enclosing item",
    skill: "handle-panic",
    parameters: Parameters::Level {
        default: Severity::Error,
        violation: "an unwrap, expect, unwrap_err, expect_err, resume_unwind or panic macro aborts the program on failure; return or handle the error with ?, Result or match, or put a DECISION: or INVARIANT: reference on the line directly above it, or above an #[expect] of its Clippy lint on an enclosing item, to justify the panic",
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
// DECISION: D168
const EAGERFALLBACK: Descriptor = Descriptor {
    default_id: "eager-fallback",
    exclude: TEST_CODE,
    target: "file",
    metric: "every Rust .unwrap_or, .ok_or, .map_or or .or call whose eager first argument, outside a closure in it, propagates with ? or divides by a divisor that is not a literal, outside test code, without a DECISION: or INVARIANT: reference comment on the line directly above it",
    skill: "eager-to-lazy",
    parameters: Parameters::Level {
        default: Severity::Error,
        violation: "the argument of unwrap_or, ok_or, map_or or or is evaluated unconditionally, so a ? inside it propagates and a division can panic even when the value is present; move it into the lazy unwrap_or_else, ok_or_else, map_or_else or or_else closure, or put a DECISION: or INVARIANT: reference on the line directly above it to justify the eager form",
    },
};
// DECISION: D090
// DECISION: D099
// DECISION: D114
// DECISION: D155
const DISCARDEDRESULT: Descriptor = Descriptor {
    default_id: "discarded-result",
    exclude: TEST_CODE,
    target: "file",
    metric: "every Rust bare .ok() or .err() statement that drops a Result, every bare statement whose last call is .map, .and_then, .map_or or .map_or_else and drops the combined value, every bare .status()? or .output()? statement that propagates only the launch error and drops the exit status or output of a command, and every bare statement or let _ = binding whose value, through ?, .unwrap() and .expect(..), is a call whose path or method ends in spawn, outside test code, without a DECISION: or INVARIANT: reference comment on the line directly above it",
    skill: "handle-discarded",
    parameters: Parameters::Level {
        default: Severity::Error,
        violation: "a bare .ok() or .err() statement converts a Result to Option and throws the outcome away, a bare .map, .and_then, .map_or or .map_or_else statement throws away the value the closure produced, a bare .status()? or .output()? statement keeps only the launch error and throws the exit status or output away, and a dropped spawn handle loses the thread's, task's or child's result and failure, so a failure passes silently; handle the error or the value, keep the handle and wait for it, propagate it, or put a DECISION: or INVARIANT: reference on the line directly above it to justify discarding it, such as a deliberately detached background task",
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
// DECISION: D169
const UNTARGETEDPANIC: Descriptor = Descriptor {
    default_id: "untargeted-panic",
    exclude: &[],
    target: "file",
    metric: "every Rust #[should_panic] attribute with neither a = value nor an expected argument, without a DECISION: or INVARIANT: reference comment on the line directly above it",
    skill: "expect-the-panic",
    parameters: Parameters::Level {
        default: Severity::Error,
        violation: "a should_panic test without expected passes on any panic, also one from an unrelated place, so a broken test stays green; name the expected message with #[should_panic(expected = \"...\")] or test the returned Err instead, or put a DECISION: or INVARIANT: reference on the line directly above it",
    },
};
// DECISION: D169
const TESTSUBSTITUTION: Descriptor = Descriptor {
    default_id: "test-substitution",
    exclude: TEST_CODE,
    target: "file",
    metric: "every Rust #[cfg(..)] or #![cfg(..)] attribute of an item in a file, module, impl or trait body whose predicate is false in every build with test set, such as not(test), all(not(test), ..) or not(any(test, ..)), outside test code, without a DECISION: or INVARIANT: reference comment on the line directly above it",
    skill: "inject-the-variant",
    parameters: Parameters::Level {
        default: Severity::Error,
        violation: "an item compiled only outside tests never runs under test, and a cfg(test) twin makes the tests check other code than the product runs; pass what varies as a parameter or trait value that the caller and the test both supply, or put a DECISION: or INVARIANT: reference on the line directly above it",
    },
};
const INTERPOLATEDSHELL: Descriptor = Descriptor {
    default_id: "interpolated-shell",
    exclude: TEST_CODE,
    target: "file",
    metric: "every Rust Command::new call whose first argument is a string literal naming sh, bash, zsh, dash, ksh, fish, cmd, powershell or pwsh by its last path component, with or without .exe, whose chained .arg and .args([..]) arguments put a -c, /c, /C, -Command or lowercase short-flag cluster ending in c directly before an argument that is neither a string literal nor an uppercase constant name, outside test code, without a DECISION: or INVARIANT: reference comment on the line directly above the call",
    skill: "pass-arguments-apart",
    parameters: Parameters::Level {
        default: Severity::Error,
        violation: "a value substituted into a shell command string is parsed as shell code, so a space, ; or $( ) in a path or name changes or extends the command; run the program directly with each value as its own argument, pass values as positional parameters after the script, or put a DECISION: or INVARIANT: reference on the line directly above the call",
    },
};
const INTERPOLATEDSYNTAX: Descriptor = Descriptor {
    default_id: "interpolated-syntax",
    exclude: TEST_CODE,
    target: "file",
    metric: "every Rust format, write or writeln macro whose first string literal has a placeholder and either contains {{ followed by optional whitespace and a double quote or starts after leading whitespace with SELECT, INSERT, UPDATE or DELETE and whitespace, outside test code, without a DECISION: or INVARIANT: reference comment on the line directly above it",
    skill: "bind-the-value",
    parameters: Parameters::Level {
        default: Severity::Error,
        violation: "a value formatted into JSON or SQL text can carry a quote or another control character that breaks the document or rewrites the query; build JSON with a serializer and pass SQL values as bound parameters, or put a DECISION: or INVARIANT: reference on the line directly above it",
    },
};
const STALECHECK: Descriptor = Descriptor {
    default_id: "stale-check",
    exclude: TEST_CODE,
    target: "file",
    metric: "every Rust exists, is_file or is_dir method call without arguments followed later in the same fn, closures included, by a call whose path has an fs segment before its name or ends in File::open or File::create, with an argument whose text equals the checked receiver once & and .as_path() are removed from both, outside test code, without a DECISION: or INVARIANT: reference comment on the line directly above the check",
    skill: "act-and-match-the-error",
    parameters: Parameters::Level {
        default: Severity::Error,
        violation: "the path can appear, vanish or change its kind between the check and the action, so the action fails or is skipped where its own error would tell what happened; perform the action and match its error kind such as NotFound or AlreadyExists, or put a DECISION: or INVARIANT: reference on the line directly above the check",
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
