use crate::structural_lint::rule_identities::{
    ControlFlow, Kind, Ownership, Reference, Signature, Structure,
};
use serde::{Deserialize, Deserializer, de};
use serde_json::Value;

// DECISION: D133
// DECISION: D134
impl<'de> Deserialize<'de> for Kind {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let name = String::deserialize(deserializer)?;
        family(&name).ok_or_else(|| {
            let names: Vec<String> = RULE_KINDS.iter().map(|kind| format!("`{kind}`")).collect();
            de::Error::custom(format!(
                "unknown variant `{name}`, expected one of {}",
                names.join(", ")
            ))
        })
    }
}
fn family(name: &str) -> Option<Kind> {
    let value = || Value::from(name);
    serde_json::from_value(value())
        .map(Kind::Structure)
        .or_else(|_| serde_json::from_value(value()).map(Kind::ControlFlow))
        .or_else(|_| serde_json::from_value(value()).map(Kind::Signature))
        .or_else(|_| serde_json::from_value(value()).map(Kind::Ownership))
        .or_else(|_| serde_json::from_value(value()).map(Kind::Reference))
        .ok()
}

// DECISION: D016
// DECISION: D017
// DECISION: D018
pub const RULE_KINDS: &[Kind] = &[
    Kind::Structure(Structure::NonblankLines),
    Kind::Structure(Structure::LineWidth),
    Kind::Structure(Structure::ForbiddenFile),
    Kind::Structure(Structure::ProgramLanguage),
    Kind::Structure(Structure::DirectoryEntries),
    Kind::Structure(Structure::DumpName),
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
    Kind::Reference(Reference::CargoRustflags),
    Kind::ControlFlow(ControlFlow::BooleanExpression),
    Kind::Structure(Structure::EmbeddedText),
    Kind::ControlFlow(ControlFlow::NamedIfCondition),
    Kind::Reference(Reference::Comment),
    Kind::Structure(Structure::DirectoryArchitecture),
    Kind::Structure(Structure::TestPlacement),
    Kind::Structure(Structure::SameText),
    Kind::Structure(Structure::TestInCodeFile),
    Kind::Structure(Structure::FileCycle),
    Kind::Structure(Structure::EntryUse),
    Kind::Structure(Structure::EntryBypass),
];
