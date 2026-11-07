// DECISION: D033
// DECISION: D098
use super::{Analysis, Measurement, manifest, python, rust};
use crate::structural_lint::rules::control_flow::ControlFlow;
use crate::structural_lint::rules::kind::Kind;
use crate::structural_lint::rules::ownership::Ownership;
use crate::structural_lint::rules::reference::Reference;
use crate::structural_lint::rules::signature::Signature;
use crate::structural_lint::rules::structure::Structure;
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
            Kind::ControlFlow(ControlFlow::NamedIfCondition),
            Kind::Structure(Structure::FunctionLines),
            Kind::Signature(Signature::ParameterCount),
            Kind::ControlFlow(ControlFlow::NestingDepth),
            Kind::Signature(Signature::BooleanParameter),
            Kind::ControlFlow(ControlFlow::BranchBudget),
            Kind::Reference(Reference::UnsafeJustification),
            Kind::Reference(Reference::HardcodedSecret),
            Kind::Reference(Reference::SilentError),
            Kind::Reference(Reference::DisabledTest),
            Kind::Reference(Reference::AllowJustification),
            Kind::Reference(Reference::DebugArtifact),
            Kind::Reference(Reference::DebugAssertion),
            Kind::Reference(Reference::PanicPath),
            Kind::Reference(Reference::WildcardImport),
            Kind::Signature(Signature::TupleAsRecord),
            Kind::Reference(Reference::EagerFallback),
            Kind::Reference(Reference::DiscardedResult),
            Kind::Ownership(Ownership::DisabledRaii),
            Kind::Ownership(Ownership::GlobalState),
            Kind::Reference(Reference::SuperSuperReach),
            Kind::Ownership(Ownership::RecursiveDeletion),
            Kind::Reference(Reference::UnwindJustification),
            Kind::Ownership(Ownership::GuardAcrossAwait),
            Kind::Reference(Reference::BlockingInAsync),
            Kind::Signature(Signature::OrdEqMismatch),
            Kind::Signature(Signature::SecretDebug),
            Kind::Signature(Signature::UncheckedDeserialize),
            Kind::ControlFlow(ControlFlow::BooleanExpression),
            Kind::Structure(Structure::EmbeddedText),
            Kind::Reference(Reference::CommentReference),
            Kind::Structure(Structure::DirectoryArchitecture),
        ],
        inspect: Inspect::Tree(rust::inspect),
    },
    Handler {
        name: "python",
        title: "Python",
        extensions: &[".py", ".pyi"],
        rules: &[
            Kind::ControlFlow(ControlFlow::NamedIfCondition),
            Kind::Structure(Structure::FunctionLines),
            Kind::Signature(Signature::ParameterCount),
            Kind::ControlFlow(ControlFlow::NestingDepth),
            Kind::Signature(Signature::BooleanParameter),
            Kind::ControlFlow(ControlFlow::BranchBudget),
            Kind::Reference(Reference::HardcodedSecret),
            Kind::Reference(Reference::SilentError),
            Kind::Reference(Reference::DisabledTest),
            Kind::Reference(Reference::WildcardImport),
            Kind::ControlFlow(ControlFlow::BooleanExpression),
            Kind::Reference(Reference::CommentReference),
            Kind::Structure(Structure::DirectoryArchitecture),
        ],
        inspect: Inspect::Tree(python::inspect),
    },
    Handler {
        name: "toml",
        title: "TOML",
        extensions: &[".toml"],
        rules: &[Kind::Reference(Reference::AllowJustification)],
        inspect: Inspect::Document(manifest::analyze),
    },
];
pub fn handler(path: &Path) -> Option<&'static Handler> {
    let extension = format!(".{}", path.extension()?.to_str()?);
    HANDLERS
        .iter()
        .find(|handler| handler.extensions.contains(&extension.as_str()))
}
