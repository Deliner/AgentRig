use super::{Report, Request};
use crate::{digest, snapshot};
use anyhow::{Context, Result, ensure};
use std::fs;

pub fn prepare(
    request: &Request,
    report: &mut Report,
    scope: &crate::config::Repository,
) -> Result<Option<Report>> {
    let Some(path) = &request.previous_report else {
        return Ok(None);
    };
    let bytes = fs::read(path)?;
    let previous: Report = serde_json::from_slice(&bytes)?;
    ensure!(
        previous.schema_version == 1 && previous.tool == request.tool,
        "previous report belongs to another tool or schema"
    );
    ensure!(
        previous.contract_digest == report.contract_digest,
        "previous contract differs; start a new review boundary explicitly"
    );
    let old = previous
        .snapshot
        .as_ref()
        .context("previous report has no candidate snapshot")?;
    let current = report
        .snapshot
        .as_ref()
        .context("current snapshot missing")?;
    ensure!(
        old.base == current.base,
        "re-review must retain the original base"
    );
    snapshot::resolve(&request.root, &old.candidate)?;
    report.previous_report_digest = Some(digest(&bytes));
    snapshot::check_boundary(&request.root, &old.candidate, &current.candidate, scope)?;
    report.repair_diff = Some(snapshot::diff(
        &request.root,
        &old.candidate,
        &current.candidate,
    )?);
    Ok(Some(previous))
}
pub fn validate(report: &mut Report, previous: Option<&Report>) -> Result<()> {
    let Some(previous) = previous else {
        return Ok(());
    };
    let old = previous
        .snapshot
        .as_ref()
        .context("previous snapshot missing")?;
    let current = report
        .snapshot
        .as_ref()
        .context("current snapshot missing")?;
    for role in &mut report.roles {
        let Some(response) = &role.response else {
            continue;
        };
        for check in &response.checks {
            let path = check.evidence.split(':').next().unwrap_or("").trim();
            let unchanged = old.manifest.get(path).map(|entry| &entry.sha256)
                == current.manifest.get(path).map(|entry| &entry.sha256);
            let known = known(previous, check);
            let late = matches!(check.status.as_str(), "FAIL" | "BLOCKED") && unchanged && !known;
            let unexplained =
                late && (!check.late_finding || check.previous_omission.trim().is_empty());
            if unexplained {
                role.technical_error = Some(format!(
                    "late finding {} in unchanged scope needs late_finding and previous_omission",
                    check.contract_id
                ));
            }
        }
    }
    Ok(())
}

fn known(previous: &Report, check: &crate::response::Check) -> bool {
    previous
        .roles
        .iter()
        .filter_map(|role| role.response.as_ref())
        .flat_map(|response| &response.checks)
        .any(|old| {
            matches!(old.status.as_str(), "FAIL" | "BLOCKED")
                && old.contract_id == check.contract_id
                && old.evidence == check.evidence
        })
}
