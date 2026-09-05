use super::super::memory::format::{STATE_SECTIONS, columns};
use std::collections::BTreeMap;

pub fn skills() -> BTreeMap<&'static str, String> {
    BTreeMap::from([
        (
            "complexity-discipline",
            include_str!("../../../assets/skills/complexity-discipline/SKILL.md").to_owned(),
        ),
        (
            "edit-decisions",
            include_str!("../../../assets/skills/edit-decisions/SKILL.md").to_owned(),
        ),
        (
            "edit-invariants",
            include_str!("../../../assets/skills/edit-invariants/SKILL.md").to_owned(),
        ),
        (
            "edit-plan",
            include_str!("../../../assets/skills/edit-plan/SKILL.md").to_owned(),
        ),
        (
            "edit-state",
            include_str!("../../../assets/skills/edit-state/SKILL.md").to_owned(),
        ),
        (
            "execute-plan-feature",
            include_str!("../../../assets/skills/execute-plan-feature/SKILL.md").to_owned(),
        ),
        (
            "name-if-condition",
            include_str!("../../../assets/skills/name-if-condition/SKILL.md").to_owned(),
        ),
        (
            "norm-or-choice",
            include_str!("../../../assets/skills/norm-or-choice/SKILL.md").to_owned(),
        ),
        (
            "reduce-parameters",
            include_str!("../../../assets/skills/reduce-parameters/SKILL.md").to_owned(),
        ),
        (
            "refactor-large-directory",
            include_str!("../../../assets/skills/refactor-large-directory/SKILL.md").to_owned(),
        ),
        (
            "refactor-large-file",
            include_str!("../../../assets/skills/refactor-large-file/SKILL.md").to_owned(),
        ),
        (
            "refactor-long-function",
            include_str!("../../../assets/skills/refactor-long-function/SKILL.md").to_owned(),
        ),
        (
            "repair",
            include_str!("../../../assets/skills/repair/SKILL.md").to_owned(),
        ),
    ])
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
