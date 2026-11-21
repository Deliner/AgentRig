mod architecture;
mod languages;
mod lint_files;
mod reporting;
mod rule_evaluation;
mod rule_identities;
// DECISION: D007
mod rule_set;
mod targeting;
#[cfg(test)]
mod tests;

pub use architecture::{Rooted, crate_modules};
pub use lint_files::{baseline_path, only_adds_rules, only_lowers_counts};
pub use reporting::{
    Format, check, discovery, execute, failure, inspection, lint_structure, run, standalone,
};
pub use rule_evaluation::{Location, Mode, error_count, record_baseline};
pub use rule_identities::{Kind, RUST_SEED, Structure};
pub use rule_set::capabilities::{RULE_KINDS, example_rule};
pub use targeting::file_discovery::maintained_files;
