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
pub fn instructions(config: &super::Config) -> String {
    let mut instructions = format!(
        "# Project development\n\nThis repository contains the consumer project. The portable worker is installed under {}; its configuration is agentrig.yaml. Develop the project sources selected by paths.sources.\n\nRead {}/State.md and {}/Plan.md, then reconcile them with version control before resuming work. Apply the installed complexity-discipline and execute-plan-feature skills under {}. Before editing memory, read the matching edit-plan, edit-backlog, edit-decisions, edit-invariants or edit-state skill.\n\nWhen work reveals a concrete improvement outside the current task, proactively use edit-backlog to save it in the configured memory directory, then resume the task. Recording the idea needs no additional approval; it does not authorize planning or implementation. Plan contains only work explicitly selected for execution now in the current session or resumed task. Backlog stores future ideas, improvements and hypotheses without execution priority. A request to save or plan something for later belongs in Backlog. Move it to Plan only when selected to do now; automatically rotate completed Plan rows and cards into Archive/Plan.md and Archive/Plan/ through edit-plan. Required work stays in the current task.\n\nUse just run read -- COMMAND for inspection and just run write -- COMMAND for authorized changes. Ordinary file editing does not require just write. Deliver cohesive verified changes on the configured feature branch. Commits and just feature-merge run the configured gates. Follow reported repair skills without weakening project policy.\n\nReview, when enabled in agentrig.yaml, uses the configured MCP tools or just review. Source changes and acceptance remain the calling workflow's responsibility.\n",
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
