use super::super::memory::format::{STATE_SECTIONS, columns};
use std::collections::BTreeMap;

const SKILLS: &[(&str, &str)] = &[
    (
        "complexity-discipline",
        include_str!("../../../assets/skills/complexity-discipline/SKILL.md"),
    ),
    (
        "edit-decisions",
        include_str!("../../../assets/skills/edit-decisions/SKILL.md"),
    ),
    (
        "edit-invariants",
        include_str!("../../../assets/skills/edit-invariants/SKILL.md"),
    ),
    (
        "edit-plan",
        include_str!("../../../assets/skills/edit-plan/SKILL.md"),
    ),
    (
        "edit-state",
        include_str!("../../../assets/skills/edit-state/SKILL.md"),
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
        include_str!("../../../assets/skills/norm-or-choice/SKILL.md"),
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
pub fn skills() -> BTreeMap<&'static str, String> {
    SKILLS
        .iter()
        .map(|(name, source)| (*name, (*source).to_owned()))
        .collect()
}
pub fn memory() -> BTreeMap<&'static str, String> {
    let mut files = BTreeMap::new();
    for (name, prefix) in [("Plan", 'P'), ("Decisions", 'D'), ("Invariants", 'I')] {
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
