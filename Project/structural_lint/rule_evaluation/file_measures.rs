use crate::shared::fs_with_context as fs;
use crate::structural_lint::rule_identities::{CONTRACT_FILENAME, Kind, Structure};
use crate::structural_lint::rule_set::lint_config;
use crate::structural_lint::targeting::{file_discovery, lint_targets};
use anyhow::Result;
use std::path::Path;

// DECISION: D072
const DUMP_NAMES: &[&str] = &["utils", "helpers", "misc", "common"];
// DECISION: D036
pub(super) fn structural_measurements(
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
