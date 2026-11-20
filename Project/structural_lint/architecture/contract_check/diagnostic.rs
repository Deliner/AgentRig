use crate::shared::diagnostics::actionable_findings::Diagnostic;
use crate::structural_lint::architecture::Issue;
use crate::structural_lint::rule_set::lint_config::Rule;
use anyhow::Result;

pub(super) fn diagnostic(rule: &Rule, issue: Issue) -> Result<Diagnostic> {
    let level = rule
        .level
        .ok_or_else(|| anyhow::anyhow!("architecture rule without a level"))?;
    Ok(Diagnostic {
        rule: rule.id.clone(),
        path: issue.path.to_string_lossy().into_owned(),
        level,
        actual: None,
        limit: None,
        skill: rule.skill(level),
        message: issue.message,
        line: issue.line,
        symbol: None,
        rerun: String::new(),
    })
}
