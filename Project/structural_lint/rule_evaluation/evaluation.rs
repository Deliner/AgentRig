use super::file_measures::{structural_measurements, syntax_analyses};
use super::findings::{finding, mixed_junction, syntax_error};
use super::lifting;
use crate::shared::diagnostics::Diagnostic;
use crate::shared::fs_with_context as fs;
use crate::shared::paths::globs;
use crate::structural_lint::architecture;
use crate::structural_lint::languages::{self, program_language};
use crate::structural_lint::rule_identities::{ControlFlow, Kind, Pair, Reference, Structure};
use crate::structural_lint::rule_set::{capabilities, lint_config};
use crate::structural_lint::targeting::{file_discovery, lint_targets};
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
    let inventory = file_discovery::maintained_inventory(root, &globs(&config.exclude)?)?;
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
    let targets = lint_targets::select(rule, inventory)?;
    let syntax_rule = capabilities::syntax(rule.kind);
    if syntax_rule {
        return syntax_rule_findings(root, rule, &targets, evaluation);
    }
    for selected in &targets {
        for (line, actual) in structural_measurements(root, rule, selected, inventory)? {
            if let Some(mut item) = finding(rule, selected, actual)? {
                item.line = line;
                evaluation.output.push(item);
            }
        }
    }
    Ok(())
}
fn syntax_rule_findings(
    root: &Path,
    rule: &lint_config::Rule,
    targets: &[lint_targets::Selected<'_>],
    evaluation: &mut Evaluation<'_>,
) -> Result<()> {
    let Evaluation {
        analyses,
        output,
        pairs,
        ..
    } = evaluation;
    let unparsed: Vec<&Path> = targets
        .iter()
        .map(|selected| selected.path)
        .filter(|path| !analyses.contains_key(*path))
        .collect();
    let mut parsed = syntax_analyses(root, &unparsed, pairs)?;
    for selected in targets {
        let path = selected.path;
        let analysis = match analyses.entry(path.to_owned()) {
            Entry::Occupied(place) => place.into_mut(),
            Entry::Vacant(place) => {
                let result = parsed
                    .remove(path)
                    .context("lint analyzes each selected file before measuring it")?;
                place.insert(result?)
            }
        };
        syntax_findings(rule, selected, analysis, output)?;
    }
    Ok(())
}
// DECISION: D198
const SHEBANG_BYTES: u64 = 256;
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
