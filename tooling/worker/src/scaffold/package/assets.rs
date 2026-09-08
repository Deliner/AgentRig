use super::super::memory::format::{STATE_SECTIONS, columns};
use std::collections::BTreeMap;

const SKILLS: &[(&str, &str)] = &[
    (
        "edit-backlog",
        include_str!("../../../assets/skills/memory/edit-backlog/SKILL.md"),
    ),
    (
        "complexity-discipline",
        include_str!("../../../assets/skills/complexity-discipline/SKILL.md"),
    ),
    (
        "edit-decisions",
        include_str!("../../../assets/skills/memory/edit-decisions/SKILL.md"),
    ),
    (
        "edit-invariants",
        include_str!("../../../assets/skills/memory/edit-invariants/SKILL.md"),
    ),
    (
        "edit-plan",
        include_str!("../../../assets/skills/memory/edit-plan/SKILL.md"),
    ),
    (
        "edit-state",
        include_str!("../../../assets/skills/memory/edit-state/SKILL.md"),
    ),
    (
        "execute-plan-feature",
        include_str!("../../../assets/skills/execute-plan-feature/SKILL.md"),
    ),
    (
        "name-if-condition",
        include_str!("../../../assets/skills/name-if-condition/SKILL.md"),
    ),
    (
        "norm-or-choice",
        include_str!("../../../assets/skills/memory/norm-or-choice/SKILL.md"),
    ),
    (
        "reduce-parameters",
        include_str!("../../../assets/skills/reduce-parameters/SKILL.md"),
    ),
    (
        "refactor-large-directory",
        include_str!("../../../assets/skills/refactor-large-directory/SKILL.md"),
    ),
    (
        "refactor-large-file",
        include_str!("../../../assets/skills/refactor-large-file/SKILL.md"),
    ),
    (
        "refactor-long-function",
        include_str!("../../../assets/skills/refactor-long-function/SKILL.md"),
    ),
    (
        "repair",
        include_str!("../../../assets/skills/repair/SKILL.md"),
    ),
];
pub fn skills(config: &super::Config) -> BTreeMap<&'static str, String> {
    let mut skills: BTreeMap<_, _> = SKILLS
        .iter()
        .map(|(name, source)| (*name, (*source).to_owned()))
        .collect();
    for (capability, name, source) in [
        (
            &config.capabilities.review,
            "review-project",
            include_str!("../../../assets/skills/review-project/SKILL.md"),
        ),
        (
            &config.capabilities.delegation,
            "delegate-task",
            include_str!("../../../assets/skills/delegate-task/SKILL.md"),
        ),
    ] {
        let enabled = capability.is_some();
        if enabled {
            skills.insert(name, source.to_owned());
        }
    }
    skills
}
pub fn memory() -> BTreeMap<&'static str, String> {
    let mut files = BTreeMap::new();
    for (name, prefix) in [
        ("Plan", 'P'),
        ("Backlog", 'B'),
        ("Decisions", 'D'),
        ("Invariants", 'I'),
    ] {
        let columns = columns(prefix).join(" | ");
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
    files
}

// DECISION: D031
// DECISION: D032
pub fn instructions(config: &super::Config) -> String {
    let mut instructions = format!(
        "# Project development\n\nThis repository contains the consumer project. The portable worker is installed under {}; its configuration is agentrig.yaml. Develop the project sources selected by paths.sources.\n\nRead {}/State.md and {}/Plan.md, then reconcile them with version control before resuming work. Apply the installed complexity-discipline and execute-plan-feature skills under {}. Before editing memory, read the matching edit-plan, edit-backlog, edit-decisions, edit-invariants or edit-state skill.\n\nUse edit-backlog to draft new ideas and requirement changes in chat; save the shown version only after explicit user agreement. All new product outcomes pass through Backlog before Plan. Plan contains only agreed work selected for current or resumed execution, referencing the same acceptance contract. A selected set runs autonomously without repeated questions about implementation or permission between cards. Backlog has no execution priority. Follow the canonical skills for unresolved behavior, criterion-based acceptance and automatic rotation of completed Plan rows/cards into Archive/Plan.md and Archive/Plan/. Required work remains in the current task.\n\nUse just run read -- COMMAND for inspection and just run write -- COMMAND for authorized changes. Ordinary file editing does not require just write. Deliver cohesive verified changes on the configured feature branch. Commits and just feature-merge run the configured gates. Follow reported repair skills without weakening project policy.\n\nReview, when enabled in agentrig.yaml, uses the configured MCP tools or just review. Source changes and acceptance remain the calling workflow's responsibility.\n",
        config.paths.service, config.paths.memory, config.paths.memory, config.paths.skills
    );
    let delegation = config.capabilities.delegation.is_some();
    if delegation {
        instructions.push_str(&format!(
            "\nFor delegated tasks, read {}/delegate-task/SKILL.md and use the configured worker_delegation MCP tools or just delegate. Keep run_id and owner when recovering the task.\n",
            config.paths.skills
        ));
    }
    instructions
}
