// DECISION: D033
// DECISION: D098
// DECISION: D223
use super::neutral_metrics::{Analysis, Code};
use super::{flag_list, manifest, python, rust, test_files};
use crate::structural_lint::rule_identities::{
    ControlFlow, Kind, Ownership, Pair, Reference, Signature, Structure,
};
use std::path::Path;
use tree_sitter::Node;

pub enum Inspect {
    Tree(fn(Node<'_>, Code<'_>, &[Pair], &mut Analysis)),
    Document(fn(&str) -> Analysis),
}
pub struct Handler {
    pub name: &'static str,
    pub title: &'static str,
    pub extensions: &'static [&'static str],
    pub rules: &'static [Kind],
    pub inspect: Inspect,
    pub test_file: Option<fn(&str) -> bool>,
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
            Kind::Signature(Signature::ImplicitOrder),
            Kind::Ownership(Ownership::UnitLock),
            Kind::ControlFlow(ControlFlow::UnlabeledExit),
            Kind::ControlFlow(ControlFlow::LengthSubtraction),
            Kind::Signature(Signature::SameTypeParameters),
            Kind::Signature(Signature::SameTypePositions),
            Kind::Signature(Signature::PrimitiveAlias),
            Kind::Signature(Signature::AmbiguousOption),
            Kind::Signature(Signature::PartialImpl),
            Kind::Signature(Signature::ConstantDefault),
            Kind::Ownership(Ownership::OpenCheckedField),
            Kind::Ownership(Ownership::WrapperDeref),
            Kind::Ownership(Ownership::PrimitiveFrom),
            Kind::Ownership(Ownership::PairedCalls),
            Kind::ControlFlow(ControlFlow::VariantEquality),
            Kind::ControlFlow(ControlFlow::RangeChain),
            Kind::Reference(Reference::PositionalFormat),
            Kind::Reference(Reference::CrossedField),
            Kind::Reference(Reference::ParallelIndex),
            Kind::Signature(Signature::PositionalFlag),
            Kind::Ownership(Ownership::ExposedGuard),
            Kind::Reference(Reference::UntargetedPanic),
            Kind::Reference(Reference::TestSubstitution),
            Kind::Reference(Reference::InterpolatedShell),
            Kind::ControlFlow(ControlFlow::HiddenExit),
            Kind::ControlFlow(ControlFlow::StaleCheck),
            Kind::Signature(Signature::UntypedConversion),
            Kind::Reference(Reference::SupersededForm),
            Kind::Reference(Reference::EpsilonTolerance),
            Kind::Signature(Signature::TraitPathCall),
            Kind::ControlFlow(ControlFlow::VariantList),
            Kind::Reference(Reference::DottedExtension),
            Kind::Reference(Reference::IndexedArrayLiteral),
            Kind::Reference(Reference::OuterFlagFilter),
            Kind::Reference(Reference::MaxFallback),
            Kind::ControlFlow(ControlFlow::RepeatedClassification),
            Kind::Reference(Reference::FixedTestPort),
            Kind::ControlFlow(ControlFlow::BooleanExpression),
            Kind::Structure(Structure::EmbeddedText),
            Kind::Reference(Reference::Comment),
            Kind::Structure(Structure::DirectoryArchitecture),
            Kind::Structure(Structure::TestPlacement),
            Kind::Structure(Structure::TestInCodeFile),
        ],
        inspect: Inspect::Tree(rust::inspect),
        test_file: Some(test_files::rust),
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
            Kind::Reference(Reference::Comment),
            Kind::Structure(Structure::DirectoryArchitecture),
            Kind::Structure(Structure::TestPlacement),
        ],
        inspect: Inspect::Tree(python::inspect),
        test_file: Some(test_files::python),
    },
    Handler {
        name: "toml",
        title: "TOML",
        extensions: &[".toml"],
        rules: &[
            Kind::Reference(Reference::AllowJustification),
            Kind::Reference(Reference::CargoRustflags),
        ],
        inspect: Inspect::Document(manifest::analyze),
        test_file: None,
    },
    // DECISION: D182
    Handler {
        name: "yaml",
        title: "YAML",
        extensions: &[".yaml"],
        rules: &[Kind::Reference(Reference::AllowJustification)],
        inspect: Inspect::Document(flag_list::analyze),
        test_file: None,
    },
];
pub fn handler(path: &Path) -> Option<&'static Handler> {
    let extension = format!(".{}", path.extension()?.to_str()?);
    HANDLERS
        .iter()
        .find(|handler| handler.extensions.contains(&extension.as_str()))
}
