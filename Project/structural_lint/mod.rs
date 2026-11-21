pub mod architecture;
mod languages;
mod lint_files;
mod reporting;
mod rule_evaluation;
pub mod rule_identities;
// DECISION: D007
pub mod rule_set;
pub mod targeting;
#[cfg(test)]
mod tests;

pub use lint_files::{baseline_path, only_adds_rules, only_lowers_counts};
pub use reporting::{
    Format, check, discovery, execute, failure, inspection, lint_structure, run, standalone,
};
pub use rule_evaluation::{Location, Mode, error_count, record_baseline};
