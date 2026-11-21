use crate::shared::source_parsers::Exception;
use crate::structural_lint::rule_identities::{Kind, Pair};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use tree_sitter::Node;

// DECISION: D229
pub trait Project: Sync {
    fn foreign_crate(&self, name: &str) -> bool;
    fn python_import(&self, from: &Path, import: PythonImport<'_>) -> Imported;
}
// DECISION: D229
#[derive(Clone, Copy)]
pub struct PythonImport<'a> {
    pub module: &'a str,
    pub name: &'a str,
}
// DECISION: D229
pub enum Imported {
    Outside,
    Inside(Vec<Source>),
    Unknown,
}
pub struct Source {
    pub path: PathBuf,
    pub text: String,
}
// DECISION: D156
// DECISION: D229
#[derive(Clone, Copy)]
pub struct Context<'a> {
    pub pairs: &'a [Pair],
    pub project: &'a dyn Project,
}

// DECISION: D017
pub struct Measurement {
    pub kind: Kind,
    pub line: usize,
    pub symbol: String,
    pub actual: usize,
}
#[derive(Default)]
pub struct Analysis {
    pub measurements: Vec<Measurement>,
    // DECISION: D156
    pub mixed_junctions: Vec<usize>,
    pub parse_error: Option<usize>,
    // DECISION: D222
    pub exceptions: Vec<Exception>,
    pub lifted: BTreeSet<usize>,
}
#[derive(Clone, Copy)]
pub struct Code<'a> {
    path: &'a Path,
    text: &'a str,
}
impl<'a> Code<'a> {
    pub fn new(path: &'a Path, text: &'a str) -> Self {
        Self { path, text }
    }
    pub fn path(self) -> &'a Path {
        self.path
    }
    pub fn as_str(self) -> &'a str {
        self.text
    }
}
// DECISION: D127 allow-justification
#[expect(clippy::string_slice)]
pub fn text<'a>(node: Node<'_>, source: Code<'a>) -> &'a str {
    &source.text[node.byte_range()]
}
