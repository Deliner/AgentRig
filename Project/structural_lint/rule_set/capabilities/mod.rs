mod describe;
mod language_support;

pub use crate::structural_lint::rule_identities::kind::Kind;
pub use crate::structural_lint::rule_identities::{Descriptor, Parameters};
pub use describe::{catalog, describe, example};
pub use language_support::{
    support, supports_path, syntax, validate_extensions, validate_includes,
};

use crate::structural_lint::rule_identities::control_flow::ControlFlow;
use crate::structural_lint::rule_identities::ownership::Ownership;
use crate::structural_lint::rule_identities::reference::Reference;
use crate::structural_lint::rule_identities::signature::Signature;
use crate::structural_lint::rule_identities::structure::Structure;
use serde::{Deserialize, Deserializer, de};
use serde_json::Value;

// DECISION: D133
// DECISION: D134
impl<'de> Deserialize<'de> for Kind {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let name = String::deserialize(deserializer)?;
        family(&name).ok_or_else(|| {
            let names: Vec<String> = ALL.iter().map(|kind| format!("`{kind}`")).collect();
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
pub const ALL: &[Kind] = &[
    Kind::Structure(Structure::NonblankLines),
    Kind::Structure(Structure::LineWidth),
    Kind::Structure(Structure::ForbiddenFile),
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
    Kind::Signature(Signature::PositionalFlag),
    Kind::Ownership(Ownership::ExposedGuard),
    Kind::Reference(Reference::UntargetedPanic),
    Kind::Reference(Reference::TestSubstitution),
    Kind::Reference(Reference::InterpolatedShell),
    Kind::ControlFlow(ControlFlow::HiddenExit),
    Kind::ControlFlow(ControlFlow::StaleCheck),
    Kind::Signature(Signature::UntypedConversion),
    Kind::Reference(Reference::SupersededForm),
    Kind::ControlFlow(ControlFlow::BooleanExpression),
    Kind::Structure(Structure::EmbeddedText),
    Kind::ControlFlow(ControlFlow::NamedIfCondition),
    Kind::Reference(Reference::Comment),
    Kind::Structure(Structure::DirectoryArchitecture),
    Kind::Structure(Structure::TestPlacement),
];
