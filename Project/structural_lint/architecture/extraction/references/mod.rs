mod recorder;
mod reference;

use crate::structural_lint::architecture::Issue;
pub(super) use recorder::Source;
pub use reference::{Reference, RustMacro, Target};

#[derive(Default, Debug)]
pub struct References {
    pub items: Vec<Reference>,
    pub issues: Vec<Issue>,
    pub rust_items: Vec<RustItem>,
    pub rust_imports: Vec<RustImport>,
    pub rust_macros: Vec<RustItem>,
    pub rust_generated: Vec<Vec<String>>,
    pub propagated_calls: Vec<PropagatedCall>,
}

// DECISION: D125
#[derive(Debug)]
pub struct PropagatedCall {
    pub method: String,
    pub line: usize,
}

#[derive(Debug)]
pub struct RustItem {
    pub name: String,
    pub scope: Vec<String>,
}

#[derive(Debug)]
pub struct RustImport {
    pub name: String,
    pub path: String,
    pub scope: Vec<String>,
}
