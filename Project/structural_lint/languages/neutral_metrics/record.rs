use crate::structural_lint::rule_identities::kind::Kind;
use tree_sitter::Node;

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
}
#[derive(Clone, Copy)]
pub struct Code<'a> {
    text: &'a str,
}
impl<'a> Code<'a> {
    pub fn new(text: &'a str) -> Self {
        Self { text }
    }
    pub fn as_str(self) -> &'a str {
        self.text
    }
}
// DECISION: D127
#[expect(clippy::string_slice)]
pub fn text<'a>(node: Node<'_>, source: Code<'a>) -> &'a str {
    &source.text[node.byte_range()]
}
