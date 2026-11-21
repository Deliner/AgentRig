use crate::structural_lint::languages::analyze;
use crate::structural_lint::rule_identities::{Kind, Ownership};
use std::path::Path;

// DECISION: D096
fn global_findings(path: &str, source: &str) -> Vec<String> {
    let analysis = analyze(Path::new(path), source, &[]).unwrap();
    assert_eq!(analysis.parse_error, None);
    let mut found: Vec<_> = analysis
        .measurements
        .into_iter()
        .filter(|item| item.kind == Kind::Ownership(Ownership::GlobalState))
        .collect();
    found.sort_by_key(|item| item.line);
    found
        .into_iter()
        .map(|item| format!("{}:{}", item.line, item.symbol))
        .collect()
}

#[test]
fn rust_flags_static_locks_and_public_cells() {
    let source = "use std::sync::Mutex;
static REGISTRY: Mutex<Vec<u8>> = Mutex::new(Vec::new());
pub static COUNTER: AtomicUsize = AtomicUsize::new(0);
static PRIVATE: AtomicUsize = AtomicUsize::new(0);
static LAZY: LazyLock<RwLock<u8>> = LazyLock::new(|| RwLock::new(0));
static ONCE: OnceLock<String> = OnceLock::new();
static mut RAW: u8 = 0;
pub(crate) static SHARED: std::sync::Arc<parking_lot::RwLock<u8>> = Arc::new(RwLock::new(0));
type Registry = Mutex<Vec<u8>>;
type Shared = LazyLock<Registry>;
static ALIASED: Shared = LazyLock::new(|| Mutex::new(Vec::new()));
static TEXT: &str = \"static X: Mutex<u8>\";
struct Holder {
    lock: Mutex<u8>,
}
#[cfg(test)]
mod tests {
    static TEST_LOCK: Mutex<u8> = Mutex::new(0);
}
";
    assert_eq!(
        global_findings("source.rs", source),
        [
            "2:REGISTRY",
            "3:COUNTER",
            "5:LAZY",
            "8:SHARED",
            "11:ALIASED"
        ]
    );
}

#[test]
fn rust_flags_items_declared_inside_lazy_static_and_thread_local() {
    let source = "lazy_static! {
    static ref CACHE: Mutex<u8> = Mutex::new(0);
    static ref PLAIN: String = String::new();
    pub static ref PUBLIC: AtomicBool = AtomicBool::new(false);
}
thread_local! {
    static LOCAL: RefCell<u8> = RefCell::new(0);
    pub static SHARED: RefCell<u8> = RefCell::new(0);
}
thread_local!(pub static ONE: Cell<u8> = Cell::new(0));
thread_local!(pub static TWO: Cell<u8> = Cell::new(0));
macro_rules! make {
    () => {
        static HIDDEN: Mutex<u8> = Mutex::new(0);
    };
}
#[test]
fn probe() {
    lazy_static! {
        static ref LOCK: Mutex<u8> = Mutex::new(0);
    }
}
";
    assert_eq!(
        global_findings("source.rs", source),
        ["2:CACHE", "4:PUBLIC", "8:SHARED", "10:ONE", "11:TWO"]
    );
}
