use crate::{execution::reviewer::RoleResult, snapshot::Snapshot};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, fs, path::Path};

#[derive(Debug, Deserialize, Serialize)]
pub struct Report {
    pub schema_version: u32,
    pub run_id: String,
    pub tool: String,
    pub request: serde_json::Value,
    pub snapshot: Option<Snapshot>,
    pub configuration: serde_json::Value,
    pub project_configuration: serde_json::Value,
    pub contract: serde_json::Value,
    pub contract_digest: String,
    pub response_schema: serde_json::Value,
    pub prompts: BTreeMap<String, String>,
    pub resource_digests: BTreeMap<String, String>,
    pub previous_report_digest: Option<String>,
    pub repair_diff: Option<String>,
    pub roles: Vec<RoleResult>,
    pub verdict: String,
    pub technical_error: Option<String>,
    pub cleanup_error: Option<String>,
}
impl Report {
    pub fn aggregate(&mut self, expected: usize) {
        let technical = self.technical_error.is_some()
            || self.roles.len() != expected
            || self
                .roles
                .iter()
                .any(|role| role.technical_error.is_some() || role.response.is_none());
        let failed = self
            .roles
            .iter()
            .filter_map(|role| role.response.as_ref())
            .any(|response| response.verdict == "FAIL");
        let blocked = self
            .roles
            .iter()
            .filter_map(|role| role.response.as_ref())
            .any(|response| response.verdict == "BLOCKED");
        self.verdict = if technical {
            "BLOCKED"
        } else if failed {
            "FAIL"
        } else if blocked {
            "BLOCKED"
        } else {
            "PASS"
        }
        .into();
    }
    pub fn save(&self, root: &Path) -> Result<()> {
        fs::create_dir_all(root)?;
        fs::write(
            root.join(format!("{}.json", self.run_id)),
            serde_json::to_vec_pretty(self)?,
        )?;
        let text = format!(
            "# Review {}\n\nTool: {}\n\nVerdict: {}\n\nThe complete versioned evidence, exact responses and errors follow.\n\n```json\n{}\n```\n",
            self.run_id,
            self.tool,
            self.verdict,
            serde_json::to_string_pretty(self)?
        );
        fs::write(root.join(format!("{}.md", self.run_id)), text)?;
        Ok(())
    }
}
