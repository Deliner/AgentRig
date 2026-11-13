use super::architecture::runner;
use super::languages;
use super::rule_identities;
use super::rule_set::{capabilities, lint_config};
use super::targeting::{file_discovery, lint_targets};
use crate::shared::diagnostics::actionable_findings::{Diagnostic, Level};
use crate::shared::fs_with_context as fs;
use crate::shared::paths::glob_patterns::globs;
use crate::structural_lint::rule_identities::contract_settings::FILENAME;
use crate::structural_lint::rule_identities::control_flow::MIXED_JUNCTIONS;
use crate::structural_lint::rule_identities::ownership::Pair;
use crate::structural_lint::rule_identities::structure::Structure;
use anyhow::Result;
use std::{collections::HashMap, path::Path};

#[derive(Default)]
struct Evaluation<'a> {
    analyses: HashMap<std::path::PathBuf, languages::Analysis>,
    output: Vec<Diagnostic>,
    // DECISION: D156
    pairs: &'a [Pair],
}
pub(super) fn evaluate(root: &Path, config: &lint_config::Config) -> Result<Vec<Diagnostic>> {
    let inventory = file_discovery::collect(root, &globs(&config.exclude)?)?;
    let pairs = config
        .rules
        .iter()
        .find_map(|rule| rule.pairs.as_deref())
        .unwrap_or_default();
    let mut evaluation = Evaluation {
        pairs,
        ..Evaluation::default()
    };
    for rule in &config.rules {
        let architecture =
            rule.kind == rule_identities::Kind::Structure(Structure::DirectoryArchitecture);
        if architecture {
            evaluation
                .output
                .extend(runner::run(root, rule, &inventory)?);
        } else {
            scalar_findings(root, rule, &inventory, &mut evaluation)?;
        }
    }
    Ok(evaluation.output)
}
fn scalar_findings(
    root: &Path,
    rule: &lint_config::Rule,
    inventory: &file_discovery::Inventory,
    evaluation: &mut Evaluation<'_>,
) -> Result<()> {
    let Evaluation {
        analyses,
        output,
        pairs,
    } = evaluation;
    for selected in lint_targets::select(rule, inventory)? {
        let syntax_rule = capabilities::syntax(rule.kind);
        if syntax_rule {
            let path = selected.path;
            let uncached = !analyses.contains_key(path);
            if uncached {
                let source = fs::read_to_string(root.join(path))?;
                analyses.insert(path.to_owned(), languages::analyze(path, &source, pairs)?);
            }
            syntax_findings(rule, &selected, &analyses[path], output)?;
        } else {
            for (line, actual) in structural_measurements(root, rule, &selected, inventory)? {
                if let Some(mut item) = finding(rule, &selected, actual)? {
                    item.line = line;
                    output.push(item);
                }
            }
        }
    }
    Ok(())
}
// DECISION: D072
const DUMP_NAMES: &[&str] = &["utils", "helpers", "misc", "common"];
// DECISION: D036
fn structural_measurements(
    root: &Path,
    rule: &lint_config::Rule,
    selected: &lint_targets::Selected<'_>,
    inventory: &file_discovery::Inventory,
) -> Result<Vec<(Option<usize>, usize)>> {
    let text = || -> Result<Option<String>> {
        Ok(String::from_utf8(fs::read(root.join(selected.path))?).ok())
    };
    Ok(match rule.kind {
        rule_identities::Kind::Structure(Structure::NonblankLines) => text()?
            .map(|source| nonblank_lines(&source))
            .unwrap_or_default(),
        rule_identities::Kind::Structure(Structure::LineWidth) => text()?
            .map(|source| line_widths(&source))
            .unwrap_or_default(),
        rule_identities::Kind::Structure(Structure::ForbiddenFile) => vec![(None, 1)],
        rule_identities::Kind::Structure(Structure::DirectoryEntries) => {
            vec![(None, inventory.file_entries(selected.path, FILENAME))]
        }
        rule_identities::Kind::Structure(Structure::DumpName) => dump_name(selected.path),
        rule_identities::Kind::Structure(
            Structure::FunctionLines
            | Structure::EmbeddedText
            | Structure::DirectoryArchitecture
            | Structure::TestPlacement,
        )
        | rule_identities::Kind::ControlFlow(_)
        | rule_identities::Kind::Signature(_)
        | rule_identities::Kind::Ownership(_)
        | rule_identities::Kind::Reference(_) => {
            anyhow::bail!("rule kind {} has no structural metric", rule.kind)
        }
    })
}
fn nonblank_lines(source: &str) -> Vec<(Option<usize>, usize)> {
    let count = source
        .lines()
        .filter(|line| !line.trim().is_empty())
        .count();
    vec![(None, count)]
}
fn line_widths(source: &str) -> Vec<(Option<usize>, usize)> {
    source
        .lines()
        .enumerate()
        .map(|(index, line)| (Some(index + 1), line.chars().count()))
        .collect()
}
fn dump_name(path: &Path) -> Vec<(Option<usize>, usize)> {
    let name = path.file_name().and_then(|name| name.to_str());
    let dump = name.is_some_and(|name| DUMP_NAMES.contains(&name));
    if dump { vec![(None, 1)] } else { Vec::new() }
}
fn syntax_findings(
    rule: &lint_config::Rule,
    selected: &lint_targets::Selected<'_>,
    analysis: &languages::Analysis,
    output: &mut Vec<Diagnostic>,
) -> Result<()> {
    if let Some(line) = analysis.parse_error {
        let unreported = !output
            .iter()
            .any(|item| item.rule == "syntax" && item.path == selected.path.to_string_lossy());
        if unreported {
            output.push(syntax_error(rule, selected.path, line));
        }
        return Ok(());
    }
    for measured in analysis
        .measurements
        .iter()
        .filter(|item| item.kind == rule.kind)
    {
        if let Some(mut diagnostic) = finding(rule, selected, measured.actual)? {
            diagnostic.line = Some(measured.line);
            diagnostic.symbol = Some(measured.symbol.clone());
            output.push(diagnostic);
        }
    }
    let single = !rule.junctions.mixed();
    if single {
        output.extend(
            analysis
                .mixed_junctions
                .iter()
                .map(|line| mixed_junction(rule, selected.path, *line)),
        );
    }
    Ok(())
}
// DECISION: D156
fn mixed_junction(rule: &lint_config::Rule, path: &Path, line: usize) -> Diagnostic {
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
fn syntax_error(rule: &lint_config::Rule, path: &Path, line: usize) -> Diagnostic {
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
fn finding(
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
        (None, rule_identities::Parameters::Level { violation, .. }) => (*violation).into(),
        (None, rule_identities::Parameters::Thresholds { .. }) => {
            anyhow::bail!("threshold rule reported no limit")
        }
    })
}
