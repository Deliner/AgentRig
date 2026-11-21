use super::architecture;
use super::languages::{self, program_language};
use super::lifting;
use super::rule_identities::{self, Kind};
use super::rule_set::{capabilities, lint_config};
use super::targeting::{file_discovery, lint_targets};
use crate::shared::diagnostics::{Diagnostic, Level};
use crate::shared::fs_with_context as fs;
use crate::shared::paths::globs;
use crate::structural_lint::rule_identities::{
    CONTRACT_FILENAME, ControlFlow, MIXED_JUNCTIONS, Pair, Reference, Structure,
};
use anyhow::{Context, Result};
use std::collections::{HashMap, hash_map::Entry};
use std::io::{BufRead, BufReader, Read};
use std::path::Path;

#[derive(Default)]
struct Evaluation<'a> {
    analyses: HashMap<std::path::PathBuf, languages::Analysis>,
    output: Vec<Diagnostic>,
    // DECISION: D156
    pairs: &'a [Pair],
    comment_rules: Vec<&'a lint_config::Rule>,
    // DECISION: D225
    // DECISION: D227
    // DECISION: D228
    dependency_rules: Vec<DependencyRule<'a>>,
    edges: architecture::Edges,
}
type DependencyCheck = fn(
    &lint_config::Rule,
    &architecture::Edges,
    &file_discovery::Inventory,
) -> Result<Vec<Diagnostic>>;
struct DependencyRule<'a> {
    rule: &'a lint_config::Rule,
    check: DependencyCheck,
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
        evaluation.measure(root, rule, &inventory)?;
    }
    evaluation.measure_deferred(root, &inventory)?;
    Ok(evaluation.output)
}
impl<'a> Evaluation<'a> {
    fn measure(
        &mut self,
        root: &Path,
        rule: &'a lint_config::Rule,
        inventory: &file_discovery::Inventory,
    ) -> Result<()> {
        match rule.kind {
            Kind::Reference(Reference::Comment) => self.comment_rules.push(rule),
            Kind::Structure(Structure::FileCycle) => self.defer(rule, architecture::file_cycles),
            Kind::Structure(Structure::EntryUse) => self.defer(rule, architecture::entry_uses),
            Kind::Structure(Structure::EntryBypass) => {
                self.defer(rule, architecture::entry_bypasses);
            }
            Kind::Structure(Structure::DirectoryArchitecture) => {
                let checked = architecture::run(root, rule, inventory)?;
                self.output.extend(checked.findings);
                self.edges.append(checked.edges);
            }
            Kind::Structure(Structure::ProgramLanguage) => {
                self.output.extend(foreign_programs(root, rule, inventory)?);
            }
            Kind::ControlFlow(ControlFlow::RepeatedClassification) => {
                let start = self.output.len();
                scalar_findings(root, rule, inventory, self)?;
                repeated(rule, &mut self.output, start);
            }
            Kind::Structure(_)
            | Kind::ControlFlow(_)
            | Kind::Signature(_)
            | Kind::Ownership(_)
            | Kind::Reference(_) => scalar_findings(root, rule, inventory, self)?,
        }
        Ok(())
    }
    fn defer(&mut self, rule: &'a lint_config::Rule, check: DependencyCheck) {
        self.dependency_rules.push(DependencyRule { rule, check });
    }
    // DECISION: D225
    fn measure_deferred(
        &mut self,
        root: &Path,
        inventory: &file_discovery::Inventory,
    ) -> Result<()> {
        for DependencyRule { rule, check } in std::mem::take(&mut self.dependency_rules) {
            let found = check(rule, &self.edges, inventory)?;
            self.output.extend(found);
        }
        for rule in std::mem::take(&mut self.comment_rules) {
            scalar_findings(root, rule, inventory, self)?;
            let unused = lifting::unused(rule, inventory, &self.analyses)?;
            self.output.extend(unused);
        }
        Ok(())
    }
}
// DECISION: D206
fn repeated(rule: &lint_config::Rule, output: &mut Vec<Diagnostic>, start: usize) {
    let mut first = HashMap::new();
    for mut item in output.split_off(start) {
        let measured = item.rule == rule.id;
        let Some(classification) = measured.then(|| item.symbol.clone()).flatten() else {
            output.push(item);
            continue;
        };
        match first.entry(classification) {
            Entry::Vacant(place) => {
                let line = item.line.unwrap_or_default();
                place.insert(format!("{}:{line}", item.path));
            }
            Entry::Occupied(place) => {
                let first = place.get();
                item.message = format!("{message}; first at {first}", message = item.message);
                output.push(item);
            }
        }
    }
}
// DECISION: D198
fn foreign_programs(
    root: &Path,
    rule: &lint_config::Rule,
    inventory: &file_discovery::Inventory,
) -> Result<Vec<Diagnostic>> {
    let allowed = rule.allowed_languages.as_deref().unwrap_or_default();
    let list = allowed.join(", ");
    let mut found = Vec::new();
    for selected in lint_targets::select(rule, inventory)? {
        let path = root.join(selected.path);
        let Some(name) = program_language(&path, || first_line(&path))? else {
            continue;
        };
        let foreign = !allowed.iter().any(|allowed_name| allowed_name == name);
        if foreign {
            let reported = finding(rule, &selected, 1)?;
            let mut item = reported.context("program-language reports at its level")?;
            item.message = format!(
                "{name} program; allowed languages: {list}; rewrite it in an allowed language"
            );
            found.push(item);
        }
    }
    Ok(found)
}
fn first_line(path: &Path) -> Result<Vec<u8>> {
    let mut line = Vec::new();
    BufReader::new(fs::open(path)?.take(SHEBANG_BYTES))
        .read_until(b'\n', &mut line)
        .with_context(|| format!("read the first line of {}", path.display()))?;
    Ok(line)
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
        ..
    } = evaluation;
    for selected in lint_targets::select(rule, inventory)? {
        let syntax_rule = capabilities::syntax(rule.kind);
        if syntax_rule {
            let path = selected.path;
            let analysis = match analyses.entry(path.to_owned()) {
                Entry::Occupied(place) => place.into_mut(),
                Entry::Vacant(place) => {
                    let source = fs::read_to_string(root.join(path))?;
                    place.insert(languages::analyze(path, &source, pairs)?)
                }
            };
            syntax_findings(rule, &selected, analysis, output)?;
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
// DECISION: D198
const SHEBANG_BYTES: u64 = 256;
// DECISION: D036
fn structural_measurements(
    root: &Path,
    rule: &lint_config::Rule,
    selected: &lint_targets::Selected<'_>,
    inventory: &file_discovery::Inventory,
) -> Result<Vec<(Option<usize>, usize)>> {
    let file = root.join(selected.path);
    let unmeasured = || anyhow::anyhow!("rule kind {} has no structural metric", rule.kind);
    let Kind::Structure(structure) = rule.kind else {
        return Err(unmeasured());
    };
    Ok(match structure {
        Structure::NonblankLines => text(&file)?
            .map(|source| nonblank_lines(&source))
            .unwrap_or_default(),
        Structure::LineWidth => text(&file)?
            .map(|source| line_widths(&source))
            .unwrap_or_default(),
        Structure::ForbiddenFile => vec![(None, 1)],
        Structure::DirectoryEntries => {
            vec![(
                None,
                inventory.file_entries(selected.path, CONTRACT_FILENAME),
            )]
        }
        Structure::DumpName => dump_name(selected.path),
        Structure::SameText => same_text(root, rule, selected, inventory)?,
        Structure::FunctionLines
        | Structure::ProgramLanguage
        | Structure::EmbeddedText
        | Structure::DirectoryArchitecture
        | Structure::TestPlacement
        | Structure::TestInCodeFile
        | Structure::FileCycle
        | Structure::EntryUse
        | Structure::EntryBypass => return Err(unmeasured()),
    })
}
fn text(file: &Path) -> Result<Option<String>> {
    Ok(String::from_utf8(fs::read(file)?).ok())
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
// DECISION: D195
fn same_text(
    root: &Path,
    rule: &lint_config::Rule,
    selected: &lint_targets::Selected<'_>,
    inventory: &file_discovery::Inventory,
) -> Result<Vec<(Option<usize>, usize)>> {
    let listed = lint_targets::select(rule, inventory)?;
    let first = listed.first().map_or(selected.path, |first| first.path);
    let differs = fs::read(root.join(first))? != fs::read(root.join(selected.path))?;
    Ok(if differs { vec![(None, 1)] } else { Vec::new() })
}
fn dump_name(path: &Path) -> Vec<(Option<usize>, usize)> {
    let name = path.file_name().and_then(|name| name.to_str());
    let dump = name.is_some_and(|name| DUMP_NAMES.contains(&name));
    if dump { vec![(None, 1)] } else { Vec::new() }
}
fn syntax_findings(
    rule: &lint_config::Rule,
    selected: &lint_targets::Selected<'_>,
    analysis: &mut languages::Analysis,
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
    for measured in lifting::unlifted(rule, analysis) {
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
        (None, rule_identities::Parameters::Level { violation, .. }) => {
            lifting::message(rule, violation)
        }
        (None, rule_identities::Parameters::Thresholds { .. }) => {
            anyhow::bail!("threshold rule reported no limit")
        }
    })
}
