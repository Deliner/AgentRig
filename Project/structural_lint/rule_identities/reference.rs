use super::descriptor::{Descriptor, TEST_CODE};
use super::parameters::Parameters;
use super::std_form;
use crate::shared::diagnostics::actionable_findings::Level as Severity;
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
    SupersededForm,
    RemainderSign,
    PartialOrderFallback,
    EpsilonTolerance,
    UnstableValue,
    UnstableSort,
    FrontRemoval,
    DottedExtension,
    TransmuteCall,
    UnboundedSend,
    UncheckedCall,
    ConditionalCheck,
    FixedTestPort,
    FixedTestPath,
    SilencedBinding,
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
            Self::SupersededForm => &std_form::SUPERSEDEDFORM,
            Self::RemainderSign => &std_form::REMAINDERSIGN,
            Self::PartialOrderFallback => &std_form::PARTIALORDERFALLBACK,
            Self::EpsilonTolerance => &std_form::EPSILONTOLERANCE,
            Self::UnstableValue => &std_form::UNSTABLEVALUE,
            Self::UnstableSort => &std_form::UNSTABLESORT,
            Self::FrontRemoval => &std_form::FRONTREMOVAL,
            Self::DottedExtension => &std_form::DOTTEDEXTENSION,
            Self::TransmuteCall => &TRANSMUTECALL,
            Self::UnboundedSend => &UNBOUNDEDSEND,
            Self::UncheckedCall => &UNCHECKEDCALL,
            Self::ConditionalCheck => &CONDITIONALCHECK,
            Self::FixedTestPort => &FIXEDTESTPORT,
            Self::FixedTestPath => &FIXEDTESTPATH,
            Self::SilencedBinding => &SILENCEDBINDING,
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
    metric: "every Rust call whose path ends in thread::sleep, a bare sleep imported from a thread module in the same file, or a function or method named block_on, and every call not awaited whose path, after an optional std::, starts with fs::, net::TcpStream::, net::TcpListener:: or net::UdpSocket::, inside an async fn or async block with no closure or ordinary fn between the call and that boundary, outside test code, without a DECISION: or INVARIANT: reference comment on the line directly above the call",
    skill: "await-not-block",
    parameters: Parameters::Level {
        default: Severity::Warning,
        violation: "a blocking thread::sleep, std fs or std net call or a nested block_on inside async code stalls the executor thread and freezes every task on a single-threaded runtime, and block_on panics inside a tokio runtime; await an async sleep, the future itself or the runtime's async file or socket call, move blocking work into spawn_blocking, or put a DECISION: or INVARIANT: reference on the line directly above the call to justify it",
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
// DECISION: D170
const INTERPOLATEDSHELL: Descriptor = Descriptor {
    default_id: "interpolated-shell",
    exclude: TEST_CODE,
    target: "file",
    metric: "every Rust Command::new call whose first argument is a string literal naming sh, bash, zsh, dash, ksh, fish, cmd, powershell or pwsh by its last path component, with or without .exe, whose chained .arg and .args([..]) arguments put a -c, /c, /C, -Command or lowercase short-flag cluster ending in c directly before an argument that is neither a string literal nor an uppercase constant name, outside test code, without a DECISION: or INVARIANT: reference comment on the line directly above the call",
    skill: "pass-arguments-apart",
    parameters: Parameters::Level {
        default: Severity::Error,
        violation: "a value substituted into a shell command string is parsed as shell code, so a space, ; or $( ) in a path or name changes or extends the command; run the program directly with each value as its own argument, keep the script literal and pass values apart from it, as positional parameters of a POSIX shell or fish or environment variables read by PowerShell, or put a DECISION: or INVARIANT: reference on the line directly above the call",
    },
};
const TRANSMUTECALL: Descriptor = Descriptor {
    default_id: "transmute-call",
    exclude: TEST_CODE,
    target: "file",
    metric: "every Rust call of a path ending in transmute or transmute_copy, with or without a turbofish, outside test code, without a DECISION: or INVARIANT: reference comment on the line directly above it",
    skill: "convert-checked",
    parameters: Parameters::Level {
        default: Severity::Error,
        violation: "transmute reinterprets the bits of one type as another and checks neither the validity of the value nor the layout or the lifetime, so a type changed later turns the call into undefined behavior silently; use a checked conversion such as from_bits, to_bits, from_ne_bytes, an as cast or TryFrom, or put a DECISION: or INVARIANT: reference on the line directly above it",
    },
};
const UNBOUNDEDSEND: Descriptor = Descriptor {
    default_id: "unbounded-send",
    exclude: TEST_CODE,
    target: "file",
    metric: "every Rust unsafe impl of Send or Sync with a type parameter bounded by neither Send nor Sync in its parameter list or a where clause, outside test code, without a DECISION: or INVARIANT: reference comment on the line directly above the impl",
    skill: "bound-the-parameter",
    parameters: Parameters::Level {
        default: Severity::Error,
        violation: "an unsafe impl of Send or Sync for a generic type vouches for every type parameter, also one such as Rc or Cell that must not cross or be shared between threads, and the compiler stops checking it; bound the parameter by Send or Sync as the impl requires, or put a DECISION: or INVARIANT: reference on the line directly above the impl",
    },
};
const UNCHECKEDCALL: Descriptor = Descriptor {
    default_id: "unchecked-call",
    exclude: TEST_CODE,
    target: "file",
    metric: "every Rust call of a function or method whose name ends in _unchecked or starts with unchecked_, except one with one or more arguments that are all literals and one inside a function whose name has the same form, outside test code, without a DECISION: or INVARIANT: reference comment on the line directly above it",
    skill: "use-the-checked-twin",
    parameters: Parameters::Level {
        default: Severity::Error,
        violation: "an unchecked call skips the check of its precondition, so a wrong index, length or encoding becomes undefined behavior instead of a panic or an error; call the checked twin and handle its None or Err, or put a DECISION: or INVARIANT: reference on the line directly above it naming the check that guarantees the precondition",
    },
};
const CONDITIONALCHECK: Descriptor = Descriptor {
    default_id: "conditional-check",
    exclude: &[],
    target: "file",
    metric: "every Rust function with an attribute whose path ends in test that holds at least one assert!, assert_eq! or assert_ne!, closures included, each of them in a branch of an if or if let whose chain does not end in else, without a DECISION: or INVARIANT: reference comment on the line directly above the function",
    skill: "check-unconditionally",
    parameters: Parameters::Level {
        default: Severity::Error,
        violation: "a test whose every check sits in an if without else passes without checking anything when the condition is false; assert the condition itself, bind the value with let-else and panic in the else, or put a DECISION: or INVARIANT: reference on the line directly above the function",
    },
};
const FIXEDTESTPORT: Descriptor = Descriptor {
    default_id: "fixed-test-port",
    exclude: &[],
    target: "file",
    metric: "every Rust call in test code of a path ending in TcpListener::bind or UdpSocket::bind whose argument is a string literal with a nonzero port after its last colon or a tuple whose second element is a nonzero integer literal, without a DECISION: or INVARIANT: reference comment on the line directly above it",
    skill: "bind-port-zero",
    parameters: Parameters::Level {
        default: Severity::Error,
        violation: "a fixed port is shared by every test and process on the machine, so parallel runs collide and the test fails or talks to a stranger; bind port 0 and read the assigned port with local_addr, or put a DECISION: or INVARIANT: reference on the line directly above it",
    },
};
const FIXEDTESTPATH: Descriptor = Descriptor {
    default_id: "fixed-test-path",
    exclude: &[],
    target: "file",
    metric: "every Rust call in test code of a path ending in File::create, fs::write, fs::create_dir or fs::create_dir_all whose first argument is a string literal, without a DECISION: or INVARIANT: reference comment on the line directly above it",
    skill: "write-in-a-tempdir",
    parameters: Parameters::Level {
        default: Severity::Error,
        violation: "a fixed path is shared by every test run, so parallel tests overwrite each other's files and the leftovers of a failed run change the next one; write into a fresh temporary directory owned by the test, or put a DECISION: or INVARIANT: reference on the line directly above it",
    },
};
const SILENCEDBINDING: Descriptor = Descriptor {
    default_id: "silenced-binding",
    exclude: TEST_CODE,
    target: "file",
    metric: "every Rust let _ = x; without a type where x is one identifier that is not all upper case, outside test code, without a DECISION: or INVARIANT: reference comment on the line directly above it",
    skill: "remove-the-binding",
    parameters: Parameters::Level {
        default: Severity::Error,
        violation: "let _ = x only silences the unused warning of x, so a value computed for nothing or a parameter the code forgot to use stays hidden; use the value, remove it with the code that computes it, name a parameter that a trait or callback dictates with a leading underscore, or put a DECISION: or INVARIANT: reference on the line directly above it",
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
