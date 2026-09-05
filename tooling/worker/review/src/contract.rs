use anyhow::{Result, ensure};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeSet, fs, path::Path};

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Contract {
    pub schema_version: u32,
    pub requirements: Vec<Requirement>,
}
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Requirement {
    pub id: String,
    pub text: String,
    pub reviewers: Vec<String>,
    pub allow_na: bool,
}
pub fn load(path: &Path, roles: &[String]) -> Result<Contract> {
    let contract: Contract = serde_json::from_slice(&fs::read(path)?)?;
    contract.validate(roles)?;
    Ok(contract)
}
impl Contract {
    pub fn validate(&self, roles: &[String]) -> Result<()> {
        ensure!(
            self.schema_version == 1 && !self.requirements.is_empty(),
            "contract schema 1 with requirements is required"
        );
        let mut ids = BTreeSet::new();
        for item in &self.requirements {
            ensure!(
                !item.id.trim().is_empty() && !item.text.trim().is_empty(),
                "requirement ID and text required"
            );
            ensure!(ids.insert(&item.id), "duplicate contract ID {}", item.id);
            let assigned: BTreeSet<_> = item.reviewers.iter().collect();
            ensure!(
                !assigned.is_empty() && assigned.len() == item.reviewers.len(),
                "invalid assignments for {}",
                item.id
            );
            ensure!(
                assigned.iter().all(|role| roles.contains(role)),
                "unknown assigned reviewer for {}",
                item.id
            );
        }
        for role in roles {
            ensure!(
                self.requirements
                    .iter()
                    .any(|item| item.reviewers.contains(role)),
                "reviewer {role} has no assigned requirements"
            );
        }
        Ok(())
    }
}
