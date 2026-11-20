use super::descriptor::{Descriptor, TEST_CODE};
use super::parameters::Parameters;
use crate::shared::diagnostics::Level as Severity;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

// DECISION: D133
#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum Ownership {
    DisabledRaii,
    GlobalState,
    RecursiveDeletion,
    GuardAcrossAwait,
    UnitLock,
    OpenCheckedField,
    WrapperDeref,
    PrimitiveFrom,
    PairedCalls,
    ExposedGuard,
}
impl Ownership {
    pub(super) fn descriptor(self) -> &'static Descriptor {
        match self {
            Self::DisabledRaii => &DISABLEDRAII,
            Self::GlobalState => &GLOBALSTATE,
            Self::RecursiveDeletion => &RECURSIVEDELETION,
            Self::GuardAcrossAwait => &GUARDACROSSAWAIT,
            Self::UnitLock => &UNITLOCK,
            Self::OpenCheckedField => &OPENCHECKEDFIELD,
            Self::WrapperDeref => &WRAPPERDEREF,
            Self::PrimitiveFrom => &PRIMITIVEFROM,
            Self::PairedCalls => &PAIREDCALLS,
            Self::ExposedGuard => &EXPOSEDGUARD,
        }
    }
}
// DECISION: D156
#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Pair {
    pub open: String,
    pub close: String,
}
impl Pair {
    pub fn valid(&self) -> bool {
        !self.open.is_empty() && !self.close.is_empty() && self.open != self.close
    }
}
pub fn default_pairs() -> Value {
    json!([
        {"open": "begin", "close": "commit"},
        {"open": "lock", "close": "unlock"},
        {"open": "open", "close": "close"},
        {"open": "start", "close": "stop"},
        {"open": "enter", "close": "leave"},
        {"open": "acquire", "close": "release"},
    ])
}
// DECISION: D156
const OPENCHECKEDFIELD: Descriptor = Descriptor {
    default_id: "open-checked-field",
    exclude: TEST_CODE,
    target: "file",
    metric: "every Rust struct whose checking constructor, a function without self in its inherent impl, impl TryFrom or impl FromStr returning a Result or Option of Self that refuses on a parameter, stores that parameter in a field with a visibility modifier, outside test code, reported at the struct after its attributes",
    skill: "hide-checked-field",
    parameters: Parameters::Level {
        default: Severity::Error,
        violation: "a visible field lets code outside the module build or change the value without the constructor's check; make the field private and read it through a method",
    },
    accepts_exception: true,
};
// DECISION: D156
const WRAPPERDEREF: Descriptor = Descriptor {
    default_id: "wrapper-deref",
    exclude: TEST_CODE,
    target: "file",
    metric: "every Rust impl of Deref or DerefMut for a struct of the same file whose Target is not a type parameter of that struct, outside test code, reported at the impl",
    skill: "delegate-not-deref",
    parameters: Parameters::Level {
        default: Severity::Error,
        violation: "Deref hands out every method of the inner type, including those that break the wrapper's invariant or identity; delegate the needed methods by hand",
    },
    accepts_exception: true,
};
// DECISION: D156
const PRIMITIVEFROM: Descriptor = Descriptor {
    default_id: "primitive-from",
    exclude: TEST_CODE,
    target: "file",
    metric: "every Rust impl of From with a numeric primitive, bool, char, String or &str argument for a one-field struct of the same file, outside test code, reported at the impl",
    skill: "construct-explicitly",
    parameters: Parameters::Level {
        default: Severity::Error,
        violation: "From of a raw value lets .into() build the newtype wherever the raw value stands, so the type stops telling values apart; construct it through a named function or TryFrom with a check",
    },
    accepts_exception: true,
};
// DECISION: D156
const PAIREDCALLS: Descriptor = Descriptor {
    default_id: "paired-calls",
    exclude: TEST_CODE,
    target: "file",
    metric: "every Rust block statement calling the open method of a configured pair on a receiver, followed in the same block by the pair's close method on the token-equal receiver with a ?, return, break or continue between them outside closures, outside test code",
    skill: "guard-the-pair",
    parameters: Parameters::Level {
        default: Severity::Error,
        violation: "an early exit between the opening and the closing call skips the closing one; return a guard whose Drop closes on every path",
    },
    accepts_exception: true,
};
// DECISION: D092
const DISABLEDRAII: Descriptor = Descriptor {
    default_id: "disabled-raii",
    exclude: TEST_CODE,
    target: "file",
    metric: "every Rust .keep() or .into_path() method call, or mem::forget, ManuallyDrop::new or Box::leak path call, outside test code",
    skill: "own-the-resource",
    parameters: Parameters::Level {
        default: Severity::Error,
        violation: "this call defeats RAII, so a guard's cleanup never runs and a resource outlives its owner silently; let the value drop or hold the guard for the lifetime it protects",
    },
    accepts_exception: true,
};
// DECISION: D096
const GLOBALSTATE: Descriptor = Descriptor {
    default_id: "global-state",
    exclude: TEST_CODE,
    target: "file",
    metric: "every Rust static item outside test code, excluding static mut, whose type (following same-file type aliases up to three levels) contains a lock (Mutex, RwLock, DashMap, DashSet), or a public static whose type contains any cell (also RefCell, Cell, UnsafeCell, Atomic*), including items declared inside lazy_static! or thread_local!, reported at the line where the static starts",
    skill: "own-the-state",
    parameters: Parameters::Level {
        default: Severity::Error,
        violation: "a static lock or a public static cell is process-global mutable state with no owner in the directory tree, so every writer has to be found by search; give the state an owner, a parameter, a context field or a private cell behind one owning function",
    },
    accepts_exception: true,
};
// DECISION: D102
// DECISION: D189
const RECURSIVEDELETION: Descriptor = Descriptor {
    default_id: "recursive-deletion",
    exclude: TEST_CODE,
    target: "file",
    metric: "every Rust call whose function path ends in remove_dir_all, whose bare function name is remove_dir_all or whose method is remove_dir_all, from any source such as std::fs, tokio::fs, a local wrapper or a method, in source and in macro bodies, outside test code",
    skill: "contain-the-deletion",
    parameters: Parameters::Level {
        default: Severity::Error,
        violation: "a recursive directory deletion is the most destructive file-system operation, and a wrong argument erases a working tree; delete through an owner that checks the path lies inside its own area",
    },
    accepts_exception: true,
};
// DECISION: D117
const GUARDACROSSAWAIT: Descriptor = Descriptor {
    default_id: "guard-across-await",
    exclude: TEST_CODE,
    target: "file",
    metric: "every Rust let binding of a plain name whose initializer, without an .await, ends in a .lock(), .read() or .write() call, optionally followed by ?, .unwrap(), .expect(...) or .unwrap_or_else(...), when a later statement of the same block contains an .await, also inside a macro body, before a drop of that name, outside test code, reported at the binding",
    skill: "release-the-guard",
    parameters: Parameters::Level {
        default: Severity::Warning,
        violation: "a std lock guard held across .await blocks every task that needs the lock while this one is suspended, deadlocks a single-threaded executor and makes the future !Send; drop the guard in an inner block or with drop before the .await, or use an async-aware lock",
    },
    accepts_exception: true,
};
// DECISION: D155
const UNITLOCK: Descriptor = Descriptor {
    default_id: "unit-lock",
    exclude: TEST_CODE,
    target: "file",
    metric: "every Rust Mutex<()> or RwLock<()> type from any path, and every Mutex::new(()) or RwLock::new(()) call that initializes, directly or as a call argument, a let binding without a type annotation, outside test code",
    skill: "lock-the-data",
    parameters: Parameters::Level {
        default: Severity::Error,
        violation: "a lock around () guards nothing the compiler can see, so which data it protects lives only in comments and any access without the lock compiles; move the protected data inside the lock",
    },
    accepts_exception: true,
};
// DECISION: D166
const EXPOSEDGUARD: Descriptor = Descriptor {
    default_id: "exposed-guard",
    exclude: TEST_CODE,
    target: "file",
    metric: "every Rust fn with a visibility modifier whose return type contains, at any depth, a generic type named MutexGuard, RwLockReadGuard, RwLockWriteGuard, Ref or RefMut from any path, outside test code, reported at the fn after its attributes",
    skill: "scope-the-guard",
    parameters: Parameters::Level {
        default: Severity::Error,
        violation: "a returned guard lets the caller keep the lock or RefCell borrow as long as it wants, so two calls in one expression deadlock or panic on the second borrow while the build stays clean; take a closure that receives the guarded value and return its result",
    },
    accepts_exception: true,
};
