use super::descriptor::{Descriptor, TEST_CODE};
use super::parameters::Parameters;
use crate::shared::diagnostics::lint::Level as Severity;
use serde::{Deserialize, Serialize};

// DECISION: D133
#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum Ownership {
    DisabledRaii,
    GlobalState,
    RecursiveDeletion,
    GuardAcrossAwait,
    UnitLock,
}
impl Ownership {
    pub(super) fn descriptor(self) -> &'static Descriptor {
        match self {
            Self::DisabledRaii => &DISABLEDRAII,
            Self::GlobalState => &GLOBALSTATE,
            Self::RecursiveDeletion => &RECURSIVEDELETION,
            Self::GuardAcrossAwait => &GUARDACROSSAWAIT,
            Self::UnitLock => &UNITLOCK,
        }
    }
}
// DECISION: D092
const DISABLEDRAII: Descriptor = Descriptor {
    default_id: "disabled-raii",
    exclude: TEST_CODE,
    target: "file",
    metric: "every Rust .keep() or .into_path() method call, or mem::forget, ManuallyDrop::new or Box::leak path call, outside test code without a DECISION: or INVARIANT: reference comment on the line directly above it",
    skill: "own-the-resource",
    parameters: Parameters::Level {
        default: Severity::Error,
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
        default: Severity::Error,
        violation: "a static lock or a public static cell is process-global mutable state with no owner in the directory tree, so every writer has to be found by search; give the state an owner (a parameter, a context field or a private cell behind one owning function), or put a DECISION: or INVARIANT: reference on the line directly above it to justify the global",
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
        default: Severity::Error,
        violation: "a recursive directory deletion is the most destructive file-system operation, and a wrong argument erases a working tree; delete through an owner that checks the path lies inside its own area, or put a DECISION: or INVARIANT: reference on the line directly above the call to justify the deletion",
    },
};
// DECISION: D117
const GUARDACROSSAWAIT: Descriptor = Descriptor {
    default_id: "guard-across-await",
    exclude: TEST_CODE,
    target: "file",
    metric: "every Rust let binding of a plain name whose initializer, without an .await, ends in a .lock(), .read() or .write() call, optionally followed by ?, .unwrap(), .expect(...) or .unwrap_or_else(...), when a later statement of the same block contains an .await, also inside a macro body, before a drop of that name, outside test code, without a DECISION: or INVARIANT: reference comment on the line directly above the binding",
    skill: "release-the-guard",
    parameters: Parameters::Level {
        default: Severity::Warning,
        violation: "a std lock guard held across .await blocks every task that needs the lock while this one is suspended, deadlocks a single-threaded executor and makes the future !Send; drop the guard in an inner block or with drop before the .await, use an async-aware lock, or put a DECISION: or INVARIANT: reference on the line directly above the binding to justify it",
    },
};
// DECISION: D155
const UNITLOCK: Descriptor = Descriptor {
    default_id: "unit-lock",
    exclude: TEST_CODE,
    target: "file",
    metric: "every Rust Mutex<()> or RwLock<()> type from any path, and every Mutex::new(()) or RwLock::new(()) call that initializes, directly or as a call argument, a let binding without a type annotation, outside test code, without a DECISION: or INVARIANT: reference comment on the line directly above it",
    skill: "lock-the-data",
    parameters: Parameters::Level {
        default: Severity::Error,
        violation: "a lock around () guards nothing the compiler can see, so which data it protects lives only in comments and any access without the lock compiles; move the protected data inside the lock, or put a DECISION: or INVARIANT: reference on the line directly above it when it serializes an external resource",
    },
};
