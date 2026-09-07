pub mod contract;
use anyhow::{Context, Result, ensure};
use contract::{Contract, Requirement};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeSet,
    fs::{self, OpenOptions},
    io::Read,
    os::unix::fs::OpenOptionsExt,
    path::Path,
};

pub const MAX_BYTES: u64 = 1_048_576;
pub const SCHEMA: &str = include_str!("response.json");
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Response {
    pub run_id: String,
    pub candidate: String,
    pub contract_digest: String,
    pub role: String,
    pub verdict: String,
    pub checks: Vec<Check>,
    pub observations: Vec<String>,
}
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Check {
    pub contract_id: String,
    pub status: String,
    pub evidence: String,
    pub finding: String,
    pub minimal_fix: String,
    #[serde(default)]
    pub late_finding: bool,
    #[serde(default)]
    pub previous_omission: String,
}
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Expected {
    pub run_id: String,
    pub candidate: String,
    pub contract_digest: String,
    pub role: String,
    pub contract: Contract,
}
pub fn file(path: &Path, expected: &Expected) -> Result<Response> {
    validate(&read_regular(path, MAX_BYTES)?, expected)
}
pub fn read_regular(path: &Path, max_bytes: u64) -> Result<Vec<u8>> {
    let metadata =
        fs::symlink_metadata(path).with_context(|| format!("read {}", path.display()))?;
    ensure!(
        metadata.is_file() && metadata.len() <= max_bytes,
        "response must be a regular file of at most {max_bytes} bytes"
    );
    let file = OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK)
        .open(path)?;
    ensure!(file.metadata()?.is_file(), "response is not a regular file");
    let mut bytes = Vec::new();
    file.take(max_bytes.saturating_add(1))
        .read_to_end(&mut bytes)?;
    ensure!(
        bytes.len() as u64 <= max_bytes,
        "response exceeds size limit"
    );
    Ok(bytes)
}
pub fn validate(bytes: &[u8], expected: &Expected) -> Result<Response> {
    let response = decode(bytes)?;
    ensure!(
        response.run_id == expected.run_id
            && response.candidate == expected.candidate
            && response.contract_digest == expected.contract_digest
            && response.role == expected.role,
        "response run/candidate/contract digest/role does not match this review"
    );
    let required: Vec<_> = expected
        .contract
        .requirements
        .iter()
        .filter(|item| item.reviewers.contains(&expected.role))
        .collect();
    let mut seen = BTreeSet::new();
    for check in &response.checks {
        ensure!(
            seen.insert(&check.contract_id),
            "duplicate check {}",
            check.contract_id
        );
        let item = required
            .iter()
            .find(|item| item.id == check.contract_id)
            .with_context(|| format!("unknown or unassigned requirement {}", check.contract_id))?;
        validate_check(check, item)?;
    }
    ensure!(
        seen.len() == required.len(),
        "missing assigned requirements"
    );
    ensure!(
        response.verdict == verdict(&response.checks),
        "verdict does not match checks"
    );
    Ok(response)
}
fn decode(bytes: &[u8]) -> Result<Response> {
    let value: serde_json::Value = serde_json::from_slice(bytes).context("invalid review JSON")?;
    let schema: serde_json::Value = serde_json::from_str(SCHEMA)?;
    let validator = jsonschema::validator_for(&schema)?;
    validator
        .validate(&value)
        .map_err(|error| anyhow::anyhow!("response schema: {error}"))?;
    Ok(serde_json::from_value(value)?)
}
fn validate_check(check: &Check, item: &Requirement) -> Result<()> {
    let failed = check.status == "FAIL";
    if failed {
        ensure!(
            !check.evidence.trim().is_empty()
                && !check.finding.trim().is_empty()
                && !check.minimal_fix.trim().is_empty(),
            "FAIL requires evidence, finding and minimal_fix: {}",
            item.id
        );
    }
    let na = check.status == "N/A";
    if na {
        ensure!(item.allow_na, "N/A is not allowed for {}", item.id);
    }
    if check.late_finding {
        ensure!(
            !check.previous_omission.trim().is_empty(),
            "late finding needs an explanation of the previous omission"
        );
    }
    Ok(())
}
pub fn verdict(checks: &[Check]) -> &'static str {
    let failed = checks.iter().any(|check| check.status == "FAIL");
    let blocked = checks.iter().any(|check| check.status == "BLOCKED");
    if failed {
        "FAIL"
    } else if blocked {
        "BLOCKED"
    } else {
        "PASS"
    }
}
