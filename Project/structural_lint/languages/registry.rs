// DECISION: D033
// DECISION: D098
use super::{Analysis, Measurement, manifest, python, rust};
use crate::structural_lint::rules::kind::Kind;
use std::path::Path;
use tree_sitter::Node;

pub enum Inspect {
    Tree(fn(Node<'_>, &str, &mut Vec<Measurement>)),
    Document(fn(&str) -> Analysis),
}
pub struct Handler {
    pub name: &'static str,
    pub title: &'static str,
    pub extensions: &'static [&'static str],
    pub rules: &'static [Kind],
    pub inspect: Inspect,
}
pub const HANDLERS: &[Handler] = &[
    Handler {
        name: "rust",
        title: "Rust",
        extensions: &[".rs"],
        rules: &[
            Kind::NamedIfCondition,
            Kind::FunctionLines,
            Kind::ParameterCount,
            Kind::NestingDepth,
            Kind::BooleanParameter,
            Kind::BranchBudget,
            Kind::UnsafeJustification,
            Kind::HardcodedSecret,
            Kind::SilentError,
            Kind::DisabledTest,
            Kind::AllowJustification,
            Kind::DebugArtifact,
            Kind::DebugAssertion,
            Kind::PanicPath,
            Kind::WildcardImport,
            Kind::TupleAsRecord,
            Kind::EagerFallback,
            Kind::DiscardedResult,
            Kind::DisabledRaii,
            Kind::GlobalState,
            Kind::SuperSuperReach,
            Kind::RecursiveDeletion,
            Kind::UnwindJustification,
            Kind::GuardAcrossAwait,
            Kind::BlockingInAsync,
            Kind::OrdEqMismatch,
            Kind::BooleanExpression,
            Kind::EmbeddedText,
            Kind::CommentReference,
            Kind::DirectoryArchitecture,
        ],
        inspect: Inspect::Tree(rust::inspect),
    },
    Handler {
        name: "python",
        title: "Python",
        extensions: &[".py", ".pyi"],
        rules: &[
            Kind::NamedIfCondition,
            Kind::FunctionLines,
            Kind::ParameterCount,
            Kind::NestingDepth,
            Kind::BooleanParameter,
            Kind::BranchBudget,
            Kind::HardcodedSecret,
            Kind::SilentError,
            Kind::DisabledTest,
            Kind::WildcardImport,
            Kind::BooleanExpression,
            Kind::CommentReference,
            Kind::DirectoryArchitecture,
        ],
        inspect: Inspect::Tree(python::inspect),
    },
    Handler {
        name: "toml",
        title: "TOML",
        extensions: &[".toml"],
        rules: &[Kind::AllowJustification],
        inspect: Inspect::Document(manifest::analyze),
    },
];
pub fn handler(path: &Path) -> Option<&'static Handler> {
    let extension = format!(".{}", path.extension()?.to_str()?);
    HANDLERS
        .iter()
        .find(|handler| handler.extensions.contains(&extension.as_str()))
}
