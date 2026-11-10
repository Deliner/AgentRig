use super::architecture::runner;
use super::{catalog, config, inventory, languages, rules, selection};
use crate::shared::diagnostics::lint::{Diagnostic, Level};
use crate::shared::fs;
use crate::shared::paths::patterns::globs;
use crate::structural_lint::rules::architecture::FILENAME;
use crate::structural_lint::rules::control_flow::MIXED_JUNCTIONS;
use crate::structural_lint::rules::ownership::Pair;
use crate::structural_lint::rules::structure::Structure;
use anyhow::Result;
use std::{collections::HashMap, path::Path};

#[derive(Default)]
struct Evaluation<'a> {
    analyses: HashMap<std::path::PathBuf, languages::Analysis>,
    output: Vec<Diagnostic>,
    // DECISION: D156
    pairs: &'a [Pair],
}
pub(super) fn evaluate(root: &Path, config: &config::Config) -> Result<Vec<Diagnostic>> {
    let inventory = inventory::collect(root, &globs(&config.exclude)?)?;
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
        let architecture = rule.kind == rules::Kind::Structure(Structure::DirectoryArchitecture);
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
    rule: &config::Rule,
    inventory: &inventory::Inventory,
    evaluation: &mut Evaluation<'_>,
) -> Result<()> {
    let Evaluation {
        analyses,
        output,
        pairs,
    } = evaluation;
    for selected in selection::select(rule, inventory)? {
        let syntax_rule = catalog::syntax(rule.kind);
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
    rule: &config::Rule,
    selected: &selection::Selected<'_>,
    inventory: &inventory::Inventory,
) -> Result<Vec<(Option<usize>, usize)>> {
    let text = || -> Result<Option<String>> {
        Ok(String::from_utf8(fs::read(root.join(selected.path))?).ok())
    };
    Ok(match rule.kind {
        rules::Kind::Structure(Structure::NonblankLines) => text()?
            .map(|source| nonblank_lines(&source))
            .unwrap_or_default(),
        rules::Kind::Structure(Structure::LineWidth) => text()?
            .map(|source| line_widths(&source))
            .unwrap_or_default(),
        rules::Kind::Structure(Structure::ForbiddenFile) => vec![(None, 1)],
        rules::Kind::Structure(Structure::DirectoryEntries) => {
            vec![(None, inventory.file_entries(selected.path, FILENAME))]
        }
        rules::Kind::Structure(Structure::DumpName) => dump_name(selected.path),
        rules::Kind::Structure(
            Structure::FunctionLines
            | Structure::EmbeddedText
            | Structure::DirectoryArchitecture
            | Structure::TestPlacement,
        )
        | rules::Kind::ControlFlow(_)
        | rules::Kind::Signature(_)
        | rules::Kind::Ownership(_)
        | rules::Kind::Reference(_) => {
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
    rule: &config::Rule,
    selected: &selection::Selected<'_>,
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
fn mixed_junction(rule: &config::Rule, path: &Path, line: usize) -> Diagnostic {
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
fn syntax_error(rule: &config::Rule, path: &Path, line: usize) -> Diagnostic {
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
    rule: &config::Rule,
    selected: &selection::Selected<'_>,
    actual: usize,
) -> Result<Option<Diagnostic>> {
    let (warning, error) = (selected.warning, selected.error);
    let (level, limit) = if let Some(level) = rule.level {
        (level, None)
    } else {
        config::thresholds(warning, error)?;
        match (warning, error) {
            (_, Some(limit)) if actual > limit => (Level::Error, Some(limit)),
            (Some(limit), _) if actual > limit => (Level::Warning, Some(limit)),
            _ => return Ok(None),
        }
    };
    let message = rule_message(rule, limit, actual)?;
    let blocking = level == Level::Error;
    Ok(Some(Diagnostic {
        rule: rule.id.clone(),
        path: selected.path.to_string_lossy().into_owned(),
        level,
        actual: Some(actual),
        limit,
        skill: if blocking {
            rule.error_skill.clone()
        } else {
            rule.warning_skill.clone()
        },
        message,
        line: None,
        symbol: None,
        rerun: String::new(),
    }))
}
fn rule_message(rule: &config::Rule, limit: Option<usize>, actual: usize) -> Result<String> {
    Ok(match (limit, &rule.kind.descriptor().parameters) {
        (Some(limit), _) => format!("{} measures {actual}, exceeds {limit}", rule.kind),
        (None, rules::Parameters::Level { violation, .. }) => (*violation).into(),
        (None, rules::Parameters::Thresholds { .. }) => {
            anyhow::bail!("threshold rule reported no limit")
        }
    })
}
