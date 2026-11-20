use super::languages::{Analysis, Measurement};
use super::rule_set::lint_config::Rule;
use super::targeting::{file_discovery::Inventory, lint_targets};
use crate::shared::diagnostics::Diagnostic;
use anyhow::{Context, Result};
use std::collections::HashMap;
use std::path::PathBuf;

// DECISION: D222
pub(super) fn unlifted<'a>(rule: &Rule, analysis: &'a mut Analysis) -> Vec<&'a Measurement> {
    let Analysis {
        measurements,
        exceptions,
        lifted,
        ..
    } = analysis;
    let accepts = rule.kind.descriptor().accepts_exception;
    let mut found = Vec::new();
    for measured in measurements.iter().filter(|item| item.kind == rule.kind) {
        let named = exceptions.iter().find(|exception| {
            accepts && exception.line + 1 == measured.line && exception.rule == rule.id
        });
        match named {
            Some(exception) => {
                lifted.insert(exception.line);
            }
            None => found.push(measured),
        }
    }
    found
}
// DECISION: D222
pub(super) fn message(rule: &Rule, violation: &str) -> String {
    let accepts = rule.kind.descriptor().accepts_exception;
    if accepts {
        format!(
            "{violation}, or justify it by the exception DECISION: DNNN {id} on the line directly above, listed in that decision and approved by the user",
            id = rule.id
        )
    } else {
        violation.into()
    }
}
// DECISION: D222
pub(super) fn unused(
    rule: &Rule,
    inventory: &Inventory,
    analyses: &HashMap<PathBuf, Analysis>,
) -> Result<Vec<Diagnostic>> {
    let level = rule
        .level
        .context("the comment rule reports at its level")?;
    let mut found = Vec::new();
    for selected in lint_targets::select(rule, inventory)? {
        let analysis = analyses
            .get(selected.path)
            .context("the comment rule analyzed every file it selects")?;
        let parsed = analysis.parse_error.is_none();
        let unused = analysis
            .exceptions
            .iter()
            .filter(|exception| parsed && !analysis.lifted.contains(&exception.line));
        for exception in unused {
            let decision = &exception.decision;
            let named = &exception.rule;
            found.push(Diagnostic {
                rule: rule.id.clone(),
                path: selected.path.to_string_lossy().into_owned(),
                level,
                actual: Some(1),
                limit: None,
                skill: rule.skill(level),
                message: format!(
                    "the exception {decision} {named} lifts no {named} finding on the line below; delete it and its link in the decision, or move it directly above the line it justifies"
                ),
                line: Some(exception.line),
                symbol: Some(format!("{decision} {named}")),
                rerun: String::new(),
            });
        }
    }
    Ok(found)
}
