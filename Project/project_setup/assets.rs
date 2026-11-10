use crate::project_memory::format::{Catalog, STATE_SECTIONS};
use std::collections::BTreeMap;

pub fn memory() -> anyhow::Result<BTreeMap<&'static str, String>> {
    let mut files = BTreeMap::new();
    for catalog in [
        Catalog::Plan,
        Catalog::Backlog,
        Catalog::Decisions,
        Catalog::Invariants,
    ] {
        let name = catalog.directory();
        let columns = catalog.columns().join(" | ");
        let separator = columns
            .split('|')
            .map(|_| "---")
            .collect::<Vec<_>>()
            .join(" | ");
        files.insert(
            name,
            format!("# {name}\n\n| {columns} |\n| {separator} |\n"),
        );
    }
    files.insert(
        "State",
        "# State\n\n".to_owned()
            + &STATE_SECTIONS
                .iter()
                .map(|heading| format!("## {heading}\n\nNot recorded.\n"))
                .collect::<Vec<_>>()
                .join("\n"),
    );
    Ok(files)
}

// DECISION: D031
// DECISION: D032
// DECISION: D050
// DECISION: D054
// DECISION: D110
pub fn instructions(config: &super::Config) -> String {
    let body = format!(
        "# Project development\n\nThis repository contains the consumer project. The portable worker is installed under {}; its configuration modules are .agentrig/*.yaml. Develop the project sources selected by paths.sources.\n\nRead {}/State.md and {}/Plan.md, then reconcile them with version control before resuming work. Apply the installed complexity-discipline and execute-plan-feature skills under {}. Apply type-driven-discipline when a value carries an invariant, a mode or a lifecycle, and testing-discipline whenever you add or change tests. Before editing memory, read the matching edit-plan, edit-backlog, edit-decisions, edit-invariants or edit-state skill.\n\nUse edit-backlog to draft new ideas and requirement changes in chat; save the shown version only after explicit user agreement. All new product outcomes pass through Backlog before Plan. Plan contains only agreed work selected for current or resumed execution, referencing the same acceptance contract. A selected set runs autonomously without repeated questions about implementation or permission between cards. Backlog has no execution priority. Follow the canonical skills for unresolved behavior, criterion-based acceptance and automatic rotation of completed Plan rows/cards and their Backlog cards into Archive/. Required work remains in the current task.\n\nUse just run read -- PROGRAM ARGS for inspection and just run write -- PROGRAM ARGS for authorized changes; each runs only its configured programs. Ordinary file editing does not require just write. Deliver cohesive verified changes on the configured feature branch. Commits and just feature-merge run the configured gates. Follow reported repair skills without weakening project policy.\n",
        config.paths.service, config.paths.memory, config.paths.memory, config.paths.skills
    );
    format!("{}\n", wrap(&body, 100))
}
fn wrap(text: &str, width: usize) -> String {
    text.lines()
        .map(|line| fold(line, width))
        .collect::<Vec<_>>()
        .join("\n")
}
fn fold(line: &str, width: usize) -> String {
    let mut rows: Vec<String> = Vec::new();
    for word in line.split_whitespace() {
        match rows.last_mut() {
            Some(row) if row.len() + 1 + word.len() <= width => {
                row.push(' ');
                row.push_str(word);
            }
            _ => rows.push(word.to_owned()),
        }
    }
    rows.join("\n")
}
