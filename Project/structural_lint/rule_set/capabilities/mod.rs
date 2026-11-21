mod describe;
mod kinds;
mod language_support;

pub use crate::structural_lint::rule_identities::{Kind, Parameters};
pub use describe::{catalog, describe, example_rule};
pub use kinds::RULE_KINDS;
pub use language_support::{
    support, supports_path, syntax, validate_extensions, validate_includes,
    validate_program_languages,
};
