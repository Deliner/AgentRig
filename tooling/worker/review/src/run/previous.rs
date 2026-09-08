use super::{Report, Request};
use crate::{artifacts::digest, snapshot};
use anyhow::{Context, Result, ensure};
use std::{collections::BTreeSet, fs};

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
    same_criteria(&previous, report)?;
    same_source(request, &previous, scope)?;
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
    same_contract_files(old, current)?;
    let source = scope.vcs.source(&request.root);
    source.resolve(&old.candidate)?;
    report.previous_report_digest = Some(digest(&bytes));
    snapshot::check_boundary(&request.root, &old.candidate, &current.candidate, scope)?;
    report.repair_diff = Some(source.diff(&old.candidate, &current.candidate)?);
    Ok(Some(previous))
}

fn same_criteria(previous: &Report, current: &Report) -> Result<()> {
    ensure!(
        previous.schema_version == current.schema_version && previous.tool == current.tool,
        "previous report belongs to another tool or schema"
    );
    ensure!(
        previous.contract_digest == current.contract_digest,
        "previous contract differs; start a new review boundary explicitly"
    );
    ensure!(
        previous.prompts == current.prompts,
        "previous reviewer prompts differ; start a new review boundary explicitly"
    );
    Ok(())
}

fn same_contract_files(previous: &snapshot::Snapshot, current: &snapshot::Snapshot) -> Result<()> {
    let unchanged = previous.contract_paths == current.contract_paths
        && previous.contract_paths.iter().all(|path| {
            previous.manifest.get(path).map(|entry| &entry.sha256)
                == current.manifest.get(path).map(|entry| &entry.sha256)
        });
    ensure!(
        unchanged,
        "previous normative files differ; start a new review boundary explicitly"
    );
    Ok(())
}

fn same_source(
    request: &Request,
    previous: &Report,
    scope: &crate::config::Repository,
) -> Result<()> {
    let repository: crate::config::Repository =
        serde_json::from_value(previous.project_configuration["repository"].clone())
            .context("previous report has invalid repository configuration")?;
    let root: std::path::PathBuf = serde_json::from_value(previous.request["root"].clone())
        .context("previous report has no repository root")?;
    ensure!(
        scope
            .vcs
            .matches_previous(&request.root, (&repository.vcs, &root)),
        "previous review used another VCS source; start a new review boundary explicitly"
    );
    for (before, after) in [
        (&repository.visible_paths, &scope.visible_paths),
        (&repository.contract_paths, &scope.contract_paths),
    ] {
        ensure!(
            before.iter().collect::<BTreeSet<_>>() == after.iter().collect::<BTreeSet<_>>(),
            "previous repository scope differs; start a new review boundary explicitly"
        );
    }
    Ok(())
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
