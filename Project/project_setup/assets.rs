use crate::project_memory::format::{STATE_SECTIONS, columns};
use std::collections::BTreeMap;

const SKILLS: &[(&str, &str)] = &[
    (
        "edit-backlog",
        include_str!("../shared/skills/memory/edit-backlog/SKILL.md"),
    ),
    (
        "complexity-discipline",
        include_str!("../shared/skills/complexity-discipline/SKILL.md"),
    ),
    (
        "condense-markdown",
        include_str!("../shared/skills/condense-markdown/SKILL.md"),
    ),
    (
        "develop-tools",
        include_str!("../shared/skills/develop-tools/SKILL.md"),
    ),
    (
        "edit-decisions",
        include_str!("../shared/skills/memory/edit-decisions/SKILL.md"),
    ),
    (
        "edit-invariants",
        include_str!("../shared/skills/memory/edit-invariants/SKILL.md"),
    ),
    (
        "edit-plan",
        include_str!("../shared/skills/memory/edit-plan/SKILL.md"),
    ),
    (
        "edit-state",
        include_str!("../shared/skills/memory/edit-state/SKILL.md"),
    ),
    (
        "execute-plan-feature",
        include_str!("../shared/skills/execute-plan-feature/SKILL.md"),
    ),
    (
        "flatten-nesting",
        include_str!("../shared/skills/control-flow/flatten-nesting/SKILL.md"),
    ),
    (
        "name-if-condition",
        include_str!("../shared/skills/name-if-condition/SKILL.md"),
    ),
    (
        "norm-or-choice",
        include_str!("../shared/skills/memory/norm-or-choice/SKILL.md"),
    ),
    (
        "reduce-branching",
        include_str!("../shared/skills/control-flow/reduce-branching/SKILL.md"),
    ),
    (
        "reduce-parameters",
        include_str!("../shared/skills/signature/reduce-parameters/SKILL.md"),
    ),
    (
        "refactor-large-directory",
        include_str!("../shared/skills/refactor/refactor-large-directory/SKILL.md"),
    ),
    (
        "refactor-large-file",
        include_str!("../shared/skills/refactor/refactor-large-file/SKILL.md"),
    ),
    (
        "refactor-long-function",
        include_str!("../shared/skills/refactor/refactor-long-function/SKILL.md"),
    ),
    (
        "remove-secret",
        include_str!("../shared/skills/remove-secret/SKILL.md"),
    ),
    ("repair", include_str!("../shared/skills/repair/SKILL.md")),
    (
        "replace-comment",
        include_str!("../shared/skills/references/replace-comment/SKILL.md"),
    ),
    (
        "justify-unsafe",
        include_str!("../shared/skills/references/justify-unsafe/SKILL.md"),
    ),
    (
        "handle-error",
        include_str!("../shared/skills/references/handle-error/SKILL.md"),
    ),
    (
        "justify-skip",
        include_str!("../shared/skills/references/justify-skip/SKILL.md"),
    ),
    (
        "justify-allow",
        include_str!("../shared/skills/references/justify-allow/SKILL.md"),
    ),
    (
        "remove-debug",
        include_str!("../shared/skills/references/remove-debug/SKILL.md"),
    ),
    (
        "handle-panic",
        include_str!("../shared/skills/references/handle-panic/SKILL.md"),
    ),
    (
        "firm-up-assertion",
        include_str!("../shared/skills/references/firm-up-assertion/SKILL.md"),
    ),
    (
        "narrow-import",
        include_str!("../shared/skills/references/narrow-import/SKILL.md"),
    ),
    (
        "name-the-record",
        include_str!("../shared/skills/signature/name-the-record/SKILL.md"),
    ),
    (
        "eager-to-lazy",
        include_str!("../shared/skills/references/eager-to-lazy/SKILL.md"),
    ),
    (
        "handle-discarded",
        include_str!("../shared/skills/references/handle-discarded/SKILL.md"),
    ),
    (
        "own-the-resource",
        include_str!("../shared/skills/references/own-the-resource/SKILL.md"),
    ),
    (
        "own-the-state",
        include_str!("../shared/skills/references/own-the-state/SKILL.md"),
    ),
    (
        "anchor-at-crate",
        include_str!("../shared/skills/references/anchor-at-crate/SKILL.md"),
    ),
    (
        "split-boolean-parameter",
        include_str!("../shared/skills/signature/split-boolean-parameter/SKILL.md"),
    ),
    (
        "testing-discipline",
        include_str!("../shared/skills/testing-discipline/SKILL.md"),
    ),
];
pub fn skills() -> BTreeMap<&'static str, String> {
    SKILLS
        .iter()
        .map(|(name, source)| (*name, (*source).to_owned()))
        .collect()
}
pub fn memory() -> anyhow::Result<BTreeMap<&'static str, String>> {
    let mut files = BTreeMap::new();
    for (name, prefix) in [
        ("Plan", 'P'),
        ("Backlog", 'B'),
        ("Decisions", 'D'),
        ("Invariants", 'I'),
    ] {
        let columns = columns(prefix)?.join(" | ");
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
pub fn instructions(config: &super::Config) -> String {
    let body = format!(
        "# Project development\n\nThis repository contains the consumer project. The portable worker is installed under {}; its configuration modules are .agentrig/*.yaml. Develop the project sources selected by paths.sources.\n\nRead {}/State.md and {}/Plan.md, then reconcile them with version control before resuming work. Apply the installed complexity-discipline and execute-plan-feature skills under {}. Apply testing-discipline whenever you add or change tests. Before editing memory, read the matching edit-plan, edit-backlog, edit-decisions, edit-invariants or edit-state skill.\n\nUse edit-backlog to draft new ideas and requirement changes in chat; save the shown version only after explicit user agreement. All new product outcomes pass through Backlog before Plan. Plan contains only agreed work selected for current or resumed execution, referencing the same acceptance contract. A selected set runs autonomously without repeated questions about implementation or permission between cards. Backlog has no execution priority. Follow the canonical skills for unresolved behavior, criterion-based acceptance and automatic rotation of completed Plan rows/cards and their Backlog cards into Archive/. Required work remains in the current task.\n\nUse just run read -- PROGRAM ARGS for inspection and just run write -- PROGRAM ARGS for authorized changes; each runs only its configured programs. Ordinary file editing does not require just write. Deliver cohesive verified changes on the configured feature branch. Commits and just feature-merge run the configured gates. Follow reported repair skills without weakening project policy.\n",
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
