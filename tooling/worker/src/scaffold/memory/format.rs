use anyhow::{Result, ensure};
use std::{
    collections::HashSet,
    fs,
    path::{Path, PathBuf},
};

pub const STATE_SECTIONS: &[&str] = &[
    "Focus",
    "Workspace",
    "Progress",
    "Verification",
    "Blockers",
    "Next action",
];
pub fn columns(prefix: char) -> &'static [&'static str] {
    match prefix {
        'P' => &["ID", "Status", "Depends on", "Feature", "User capability"],
        'D' => &["ID", "Decision", "Applies in"],
        'I' => &["ID", "Invariant", "Enforced by"],
        _ => unreachable!("internal memory index kind"),
    }
}

pub fn detail_sections(directory: &str) -> (&'static [&'static str], Option<&'static str>) {
    match directory {
        "Plan" => (
            &["Feature", "User capability", "Acceptance"],
            Some("Delivery"),
        ),
        "Decisions" => (
            &["Context", "Chosen", "Rejected", "Rationale", "Consequences"],
            None,
        ),
        "Invariants" => (&["Predicate", "Oracle"], None),
        _ => unreachable!("internal memory directory"),
    }
}

pub struct Row {
    pub id: String,
    pub detail: String,
    pub cells: Vec<String>,
}
pub fn link(cell: &str) -> Result<(String, String)> {
    let (name, path) = cell
        .trim()
        .strip_prefix('[')
        .and_then(|s| s.split_once("]("))
        .ok_or_else(|| anyhow::anyhow!("expected Markdown link: {cell}"))?;
    let path = path
        .strip_suffix(')')
        .ok_or_else(|| anyhow::anyhow!("incomplete link: {cell}"))?;
    Ok((name.into(), path.into()))
}
pub fn links(cell: &str) -> Vec<String> {
    let mut remaining = cell;
    let mut output = Vec::new();
    while let Some((_, tail)) = remaining.split_once("](") {
        let Some((path, rest)) = tail.split_once(')') else {
            break;
        };
        output.push(path.to_owned());
        remaining = rest;
    }
    output
}
pub fn table(path: &Path, prefix: char) -> Result<Vec<Row>> {
    let source = fs::read_to_string(path)?;
    parse_table(&source, path, prefix)
}
pub fn parse_table(source: &str, path: &Path, prefix: char) -> Result<Vec<Row>> {
    let expected = columns(prefix);
    let columns = expected.len();
    let mut ids = HashSet::new();
    let mut rows = Vec::new();
    let mut header = false;
    for line in source
        .lines()
        .filter(|line| line.trim_start().starts_with('|'))
    {
        let cells = table_cells(line, path, columns)?;
        let header_row = cells[0] == "ID";
        if header_row {
            ensure!(
                cells == expected,
                "{}: invalid table header",
                path.display()
            );
            header = true;
            continue;
        }
        let separator = cells
            .iter()
            .all(|cell| !cell.is_empty() && cell.chars().all(|c| matches!(c, '-' | ':' | ' ')));
        if separator {
            continue;
        }
        rows.push(parse_row(cells, path, prefix, &mut ids)?);
    }
    ensure!(header, "{}: missing table header", path.display());
    Ok(rows)
}
fn table_cells(line: &str, path: &Path, columns: usize) -> Result<Vec<String>> {
    let cells: Vec<String> = line
        .trim()
        .trim_matches('|')
        .split('|')
        .map(|cell| cell.trim().to_owned())
        .collect();
    ensure!(
        cells.len() == columns,
        "{}: expected {columns} columns",
        path.display()
    );
    Ok(cells)
}
fn parse_row(
    cells: Vec<String>,
    path: &Path,
    prefix: char,
    ids: &mut HashSet<String>,
) -> Result<Row> {
    let (id, detail) = link(&cells[0])?;
    ensure!(
        id.starts_with(prefix)
            && id.len() > 1
            && id[1..].chars().all(|c| c.is_ascii_digit())
            && ids.insert(id.clone()),
        "{}: invalid or duplicate ID {id}",
        path.display()
    );
    ensure!(
        cells.iter().all(|s| !s.trim().is_empty()),
        "{id}: empty table cell"
    );
    Ok(Row { id, detail, cells })
}
pub fn sections(path: &Path, required: &[&str], optional: Option<&str>) -> Result<String> {
    let source = fs::read_to_string(path)?;
    let mut headings = Vec::new();
    let mut body = false;
    for line in source.lines() {
        if let Some(heading) = line.strip_prefix("## ") {
            ensure!(
                headings.is_empty() || body,
                "{}: empty section",
                path.display()
            );
            headings.push(heading);
            body = false;
        } else {
            let section_content = !headings.is_empty() && !line.trim().is_empty();
            if section_content {
                body = true;
            }
        }
    }
    let mut extended = required.to_vec();
    if let Some(last) = optional {
        extended.push(last);
    }
    ensure!(
        body && (headings == required || headings == extended),
        "{}: sections must be {:?} followed by {:?}",
        path.display(),
        required,
        optional
    );
    Ok(source)
}
pub fn target(root: &Path, base: &Path, value: &str) -> Result<PathBuf> {
    ensure!(
        !Path::new(value).is_absolute(),
        "memory link must be relative: {value}"
    );
    let path = agentrig::paths::resolve(&base.join(value))?;
    ensure!(
        path.starts_with(root) && path.is_file(),
        "missing or escaping memory link: {value}"
    );
    Ok(path)
}
