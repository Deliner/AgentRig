use super::lifting;
use crate::shared::diagnostics::{Diagnostic, Level};
use crate::structural_lint::clippy::Message;
use crate::structural_lint::rule_identities::{self, MIXED_JUNCTIONS};
use crate::structural_lint::rule_set::lint_config;
use crate::structural_lint::targeting::lint_targets;
use anyhow::Result;
use std::path::Path;

// DECISION: D156
pub(super) fn mixed_junction(rule: &lint_config::Rule, path: &Path, line: usize) -> Diagnostic {
    Diagnostic {
        rule: rule.id.clone(),
        path: path.to_string_lossy().into_owned(),
        level: Level::Error,
        actual: None,
        limit: None,
        skill: rule.error_skill.clone(),
        message: MIXED_JUNCTIONS.into(),
        line: Some(line),
        symbol: Some("expression".into()),
        rerun: String::new(),
    }
}
pub(super) fn syntax_error(rule: &lint_config::Rule, path: &Path, line: usize) -> Diagnostic {
    Diagnostic {
        rule: "syntax".into(),
        path: path.to_string_lossy().into_owned(),
        level: Level::Error,
        actual: None,
        limit: None,
        skill: rule.error_skill.clone(),
        message: "cannot analyze malformed syntax; repair source before evaluating rules".into(),
        line: Some(line),
        symbol: None,
        rerun: String::new(),
    }
}
pub(super) fn clippy_finding(reported: Message, skill: &str) -> Diagnostic {
    Diagnostic {
        rule: reported.rule,
        path: reported.path,
        level: Level::Error,
        actual: None,
        limit: None,
        skill: skill.to_owned(),
        message: reported.text,
        line: reported.line,
        symbol: None,
        rerun: String::new(),
    }
}
pub(super) fn finding(
    rule: &lint_config::Rule,
    selected: &lint_targets::Selected<'_>,
    actual: usize,
) -> Result<Option<Diagnostic>> {
    let (warning, error) = (selected.warning, selected.error);
    let (level, limit) = if let Some(level) = rule.level {
        (level, None)
    } else {
        lint_config::Thresholds { warning, error }.validate()?;
        match (warning, error) {
            (_, Some(limit)) if actual > limit => (Level::Error, Some(limit)),
            (Some(limit), _) if actual > limit => (Level::Warning, Some(limit)),
            _ => return Ok(None),
        }
    };
    let message = rule_message(rule, limit, actual)?;
    Ok(Some(Diagnostic {
        rule: rule.id.clone(),
        path: selected.path.to_string_lossy().into_owned(),
        level,
        actual: Some(actual),
        limit,
        skill: rule.skill(level),
        message,
        line: None,
        symbol: None,
        rerun: String::new(),
    }))
}
fn rule_message(rule: &lint_config::Rule, limit: Option<usize>, actual: usize) -> Result<String> {
    Ok(match (limit, &rule.kind.descriptor().parameters) {
        (Some(limit), _) => format!("{} measures {actual}, exceeds {limit}", rule.kind),
        (None, rule_identities::Parameters::Level { violation, .. }) => {
            lifting::message(rule, violation)
        }
        (None, rule_identities::Parameters::Thresholds { .. }) => {
            anyhow::bail!("threshold rule reported no limit")
        }
    })
}
