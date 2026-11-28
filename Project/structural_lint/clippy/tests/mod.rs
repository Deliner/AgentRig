// DECISION: D240
use super::profile::{denials, skill};
use crate::shared::testing::repository;
use std::fs;

#[test]
fn each_profile_lint_leads_to_a_skill_that_names_it() {
    let skills = repository().join("Project/shared/skills");
    let rules = denials()
        .into_iter()
        .filter(|denied| denied.starts_with("clippy::"));
    let unnamed: Vec<String> = rules
        .filter(|rule| {
            let text = skill(rule, &skills).and_then(|guide| fs::read_to_string(guide).ok());
            !text.is_some_and(|text| text.contains(&format!("`{rule}`")))
        })
        .collect();
    assert!(unnamed.is_empty(), "{unnamed:?}");
}
