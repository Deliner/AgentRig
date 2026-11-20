// DECISION: D226
use super::integration;
use super::pin;
use super::state::State;
use crate::project_config::Context;
use crate::project_memory::format::{Catalog, table};
use crate::shared::atomic_files::{atomic_json, replace};
use crate::shared::console;
use crate::shared::vcs::{Repository, Revisions};
use anyhow::{Context as _, Result, ensure};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

const RECORD: &str = "consent";
const OPEN: [&str; 2] = ["pending", "active"];

#[derive(Default, Deserialize, Serialize)]
#[serde(default)]
struct Record {
    plan: Vec<String>,
    pins: Vec<Pin>,
}

#[derive(Deserialize, Serialize)]
struct Pin {
    revision: String,
    sources: Vec<String>,
}

impl Record {
    fn load(context: &Context) -> Result<Self> {
        Ok(atomic_json::load(&location(context)?)?.unwrap_or_default())
    }
    fn save(&self, context: &Context) -> Result<()> {
        atomic_json::save(&location(context)?, self)
    }
    fn add(context: &Context, revision: &str, sources: Vec<String>) -> Result<()> {
        let mut record = Self::load(context)?;
        let revision = revision.to_owned();
        record.pins.push(Pin { revision, sources });
        record.save(context)
    }
}

fn location(context: &Context) -> Result<PathBuf> {
    Ok(context.path(&context.config.paths.runtime)?.join(RECORD))
}

fn open_items(context: &Context) -> Result<Vec<String>> {
    let plan = context.path(&context.config.paths.memory)?.join("Plan.md");
    Ok(table(&plan, Catalog::Plan)?
        .into_iter()
        .filter(|row| OPEN.contains(&row.cells[1].as_str()))
        .map(|row| row.id)
        .collect())
}

pub fn give(context: &Context) -> Result<i32> {
    let plan = open_items(context)?;
    ensure!(
        !plan.is_empty(),
        "consent covers pending or active Plan items, and the Plan has none"
    );
    let mut record = Record::load(context)?;
    let items = plan.join(", ");
    record.plan = plan;
    record.save(context)?;
    console::stdout(format!(
        "Consent given: while {items} stays pending or active, or until `just revoke-consent`, propose-pin installs and pins a gated HEAD itself and `just consented-pins` lists each pin."
    ));
    Ok(0)
}

pub fn revoke(context: &Context) -> Result<i32> {
    let mut record = Record::load(context)?;
    record.plan.clear();
    record.save(context)?;
    console::stdout("Consent revoked: propose-pin prepares each pin for `just approve` again.");
    Ok(0)
}

pub fn list(context: &Context) -> Result<i32> {
    let record = Record::load(context)?;
    let none = record.pins.is_empty();
    if none {
        console::stdout("No pin was set under consent.");
    }
    for Pin { revision, sources } in &record.pins {
        let listed = pin::listed(sources);
        console::stdout(format!("{revision}\n  {listed}"));
    }
    Ok(0)
}

pub(super) fn holds(context: &Context) -> Result<bool> {
    let record = Record::load(context)?;
    let absent = record.plan.is_empty();
    if absent {
        return Ok(false);
    }
    let open = open_items(context)?;
    Ok(record.plan.iter().any(|item| open.contains(item)))
}

pub(super) fn pin(
    context: &Context,
    reference: &str,
    gate: impl FnOnce() -> Result<i32>,
) -> Result<i32> {
    let file = context
        .protection
        .pin()
        .context("propose-pin needs the setting git.pin")?;
    let repository =
        Repository::discover(&context.root)?.context("propose-pin requires a VCS repository")?;
    let revision = repository.resolve(reference)?;
    let head = repository.resolve("HEAD")?;
    ensure!(
        revision == head,
        "under consent propose-pin pins only HEAD, not {reference}"
    );
    integration::clean(&repository, &[".", &State::of(context)?.excluded()])?;
    let pinned = pin::descending(&repository, file, &revision)?;
    let code = gate()?;
    let failed = code != 0;
    if failed {
        return Ok(code);
    }
    pin::install_revision(context, &revision)?;
    let revisions = Revisions {
        from: &pinned,
        to: &revision,
    };
    let sources = pin::sources(context, &repository, revisions)?;
    let listed = pin::listed(&sources);
    Record::add(context, &revision, sources)?;
    replace(&context.root.join(file), format!("{revision}\n").as_bytes())?;
    super::approval::admit(context, &repository, file)?;
    console::stdout(format!(
        "Pinned {revision} under the user's consent; commit {file} with State.\nSource files changed since the pinned {pinned}:\n  {listed}",
        file = file.display()
    ));
    Ok(0)
}
