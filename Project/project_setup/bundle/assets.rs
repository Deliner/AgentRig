use crate::project_memory::{Catalog, GLOSSARY, NORTH_STAR, state_template};
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
    files.insert("State", state_template());
    files.insert(NORTH_STAR, "# North Star\n".into());
    files.insert(GLOSSARY, "# Glossary\n".into());
    Ok(files)
}

// DECISION: D031
// DECISION: D032
// DECISION: D050
// DECISION: D054
// DECISION: D110
// DECISION: D191
// DECISION: D195
pub fn instructions(config: &crate::project_config::Config) -> String {
    let body = format!(
        "# Project development\n\nAGENTS.md (Codex) and CLAUDE.md (Claude Code) are one text: edit both together; the same-text lint rule fails a commit when they differ.\n\nThis repository contains the consumer project. The portable worker is installed under {service}; its configuration modules are .agentrig/*.yaml. Develop the project sources selected by paths.sources.\n\nRead {memory}/State.md and {memory}/Plan.md, then reconcile them with version control before resuming work. Apply the installed complexity-discipline and execute-plan-feature skills under {skills}. Apply type-driven-discipline when a value carries an invariant, a mode or a lifecycle, and testing-discipline whenever you add or change tests. Before editing memory, read the matching edit-plan, edit-backlog, edit-decisions, edit-invariants or edit-state skill.\n\nUse edit-backlog to draft new ideas and requirement changes in chat; save the shown version only after explicit user agreement. All new product outcomes pass through Backlog before Plan. Plan contains only agreed work selected for current or resumed execution, referencing the same acceptance contract. A selected set runs autonomously without repeated questions about implementation or permission between cards. Backlog has no execution priority. Follow the canonical skills for unresolved behavior, criterion-based acceptance and automatic rotation of completed Plan rows/cards and their Backlog cards into Archive/. Required work remains in the current task.\n\nUse `just read -- PROGRAM ARGS` for inspection and `just write -- PROGRAM ARGS` for authorized changes; each runs only its configured programs. Ordinary file editing does not require just write. Deliver cohesive verified changes on the configured feature branch. Commits and `just feature-merge` run the configured gates. Follow reported repair skills without weakening project policy.\n\n## State\n\n{memory}/State.md holds only the current VAC: task, progress, checks, what is unverified and the next action. Earlier VACs live in Git history, a card's results in its Plan Delivery. Keep it within 100 nonblank lines of at most 120 characters, which lint checks. Every commit on a feature branch carries the State of its VAC. After a commit the post-commit hook empties State, a feature commit with the empty template is refused, and `just feature-merge` commits the emptied State before merging; describe the next VAC from scratch, keeping only what it needs to continue.\n\n## Commands\n\n`just --list` shows the project commands by group; `just list` prints every catalog command with its purpose. A run that repeats gets a catalog command in .agentrig/commands.yaml, run as `just run NAME`, before its second use. `just check --staged` runs every check a staged change affects in one run; do not run them one by one. Run only the checks the change and its card require, never extra confirmation runs: when fewer checks prove the same, run fewer, and after a small fix rerun only the checks covering the changed code.\n",
        service = config.paths.service,
        memory = config.paths.memory,
        skills = config.paths.skills
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
