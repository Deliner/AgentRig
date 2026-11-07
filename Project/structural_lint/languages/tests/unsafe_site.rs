use crate::structural_lint::languages::analyze;
use crate::structural_lint::rules::kind::Kind;
use std::path::Path;

// DECISION: D065
fn unsafe_findings(path: &str, source: &str) -> Vec<usize> {
    let analysis = analyze(Path::new(path), source).unwrap();
    assert_eq!(analysis.parse_error, None);
    let mut lines: Vec<_> = analysis
        .measurements
        .into_iter()
        .filter(|item| item.kind == Kind::UnsafeJustification)
        .map(|item| item.line)
        .collect();
    lines.sort();
    lines
}

#[test]
fn rust_flags_unsafe_without_a_reference_on_the_line_above() {
    let source = "fn plain() {
    let value = 1;
}
fn uses_unsafe() {
    let first = unsafe { work() };
    // DECISION: D001
    let second = unsafe { work() };
    // INVARIANT: I002
    let third = unsafe { work() };
}
unsafe fn bare() {}
// DECISION: D001
unsafe fn marked() {}
";
    assert_eq!(unsafe_findings("source.rs", source), [5, 11]);
}

// DECISION: D103
#[test]
fn rust_flags_unsafe_impl_trait_and_static_mut_through_attributes() {
    let source = "unsafe impl Send for Handle {}
// DECISION: D001
unsafe impl Sync for Handle {}
pub unsafe trait Raw {}
// INVARIANT: I002
#[cfg(unix)]
unsafe trait Marked {}
static mut COUNTER: u32 = 0;
// DECISION: D001
static mut JUSTIFIED: u32 = 0;
static PLAIN: u32 = 0;
impl Send for Safe {}
#[inline]
unsafe fn attributed() {}
// DECISION: D001
#[inline]
#[must_use]
unsafe fn stacked() -> u8 { 0 }
";
    assert_eq!(unsafe_findings("source.rs", source), [1, 4, 8, 14]);
}

// DECISION: D103
#[test]
fn rust_flags_unsafe_inside_macro_bodies() {
    let source = "macro_rules! raw {
    ($name:ident, $body:block) => {
        unsafe fn $name() { $body }
        // DECISION: D001
        unsafe fn justified() {}
        $( unsafe { $body } )*
    };
}
lazy_static! { static ref LOCK: usize = unsafe { init() }; }
fn call() { assert!(unsafe { ready() }); }
// DECISION: D001
const SAFE: usize = 1;
";
    assert_eq!(unsafe_findings("source.rs", source), [3, 6, 9, 10]);
}

// DECISION: D103
#[test]
fn rust_skips_unsafe_attribute_openers_in_macro_arguments() {
    let source = "define_class!(
    #[unsafe(super(NSObject))]
    struct Delegate;
    impl Delegate {
        #[unsafe(method(applicationDidFinishLaunching:))]
        fn did_finish_launching(&self) {
            let value = unsafe { work() };
        }
        unsafe fn raw(&self) {}
    }
);
";
    assert_eq!(unsafe_findings("source.rs", source), [7, 9]);
}

// DECISION: D103
#[test]
fn rust_skips_conditional_unsafe_attribute_openers() {
    let source = "#[cfg_attr(unix, unsafe(no_mangle))]
pub fn exported() {}
#[cfg_attr(all(unix, test), inline, unsafe(no_mangle))]
pub fn conditional() {}
define_class!(
    #[cfg_attr(unix, unsafe(super(NSObject)))]
    struct Delegate;
    impl Delegate {
        #[cfg_attr(debug_assertions, unsafe(method(run:)))]
        fn run(&self) {
            let value = unsafe { work() };
        }
        unsafe fn raw(&self) {}
    }
);
unsafe impl Send for Delegate {}
unsafe fn bare() {}
fn call() -> u8 { unsafe { work() } }
macro_rules! raw { () => { unsafe { work() } }; }
";
    assert_eq!(
        unsafe_findings("source.rs", source),
        [11, 13, 16, 17, 18, 19]
    );
}

// DECISION: D112
#[test]
fn rust_flags_unsafe_but_not_safe_items_of_unsafe_extern_blocks() {
    let source = "unsafe extern \"C\" {
    pub safe fn abs(x: i32) -> i32;
    pub unsafe fn free(pointer: *mut u8);
    safe static ERRNO: i32;
    unsafe static mut ENVIRON: *const u8;
}
";
    assert_eq!(unsafe_findings("source.rs", source), [3, 5]);
}
