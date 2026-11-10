// DECISION: D022
mod content;
pub use content::native_index;

use crate::project_config::{self, Check, Context};
use crate::shared::console;
use crate::shared::fs;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

#[derive(Serialize, Deserialize)]
struct ResultRecord {
    id: String,
    code: i32,
    skill: String,
    rerun: String,
    consecutive_failures: u64,
}
#[derive(Serialize, Deserialize)]
struct Record {
    timestamp: u64,
    revision: Option<String>,
    #[serde(default)]
    source: Source,
    fingerprint: String,
    only: Option<String>,
    #[serde(default)]
    selective: bool,
    status: String,
    code: Option<i32>,
    results: Vec<ResultRecord>,
}
pub enum Input {
    Worktree,
    Index(String),
    Revision(String),
}
// DECISION: D154
#[derive(Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
enum Source {
    #[default]
    Worktree,
    Index(String),
    Revision,
}

impl Record {
    fn exported_revision(&self) -> Option<&str> {
        match self.source {
            Source::Revision => self.revision.as_deref(),
            Source::Worktree | Source::Index(_) => None,
        }
    }

    fn index_fingerprint(&self) -> Option<&str> {
        match &self.source {
            Source::Index(saved) => Some(saved),
            Source::Worktree | Source::Revision => None,
        }
    }

    fn fingerprint(&self, context: &Context, origin: &Path) -> Result<String> {
        let tree = match self.source {
            Source::Index(_) => content::Tree::Staged,
            Source::Worktree | Source::Revision => content::Tree::Working,
        };
        match self.exported_revision() {
            Some(revision) => content::revision_fingerprint(context, origin, revision),
            None => content::fingerprint(context, origin, tree),
        }
    }
}

pub struct Attempt {
    path: PathBuf,
    record: Record,
    previous: Option<Record>,
}
impl Attempt {
    pub fn start(
        context: &Context,
        origin: &Path,
        input: Input,
        only: Option<&str>,
    ) -> Result<Self> {
        let (source, selected) = match input {
            Input::Worktree => (Source::Worktree, None),
            Input::Index(index) => (Source::Index(index), None),
            Input::Revision(revision) => (Source::Revision, Some(revision)),
        };
        let directory = project_config::relative(origin, &context.config.paths.runtime)?;
        let path = directory.join("checks.json");
        let mut record = Record {
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs(),
            revision: match selected {
                Some(revision) => Some(revision),
                None => content::head(origin)?,
            },
            source,
            fingerprint: String::new(),
            only: only.map(str::to_owned),
            selective: false,
            status: "unfinished".into(),
            code: None,
            results: Vec::new(),
        };
        record.fingerprint = record.fingerprint(context, origin)?;
        let attempt = Self {
            previous: read(&path)?,
            path,
            record,
        };
        attempt.save()?;
        Ok(attempt)
    }
    pub fn exported_revision(&self) -> Option<&str> {
        self.record.exported_revision()
    }
    pub fn selective(&mut self) -> Result<()> {
        self.record.selective = true;
        self.save()
    }
    pub fn rerun(&self, root: &Path, id: &str) -> String {
        let mut args = vec!["check".to_owned(), "--only".into(), id.into()];
        let staged = self.record.index_fingerprint().is_some();
        if staged {
            args.push("--staged".into());
        }
        if let Some(revision) = self.exported_revision() {
            args.extend(["--revision".into(), revision.into()]);
        }
        crate::shared::diagnostics::rerun(root, &args)
    }
    pub fn checked(&mut self, check: &Check, code: i32, rerun: String) -> Result<()> {
        let prior = self
            .previous
            .as_ref()
            .and_then(|record| record.results.iter().find(|result| result.id == check.id));
        let failed = code != 0;
        let consecutive_failures = if failed {
            prior.map_or(1, |result| result.consecutive_failures.saturating_add(1))
        } else {
            0
        };
        self.record.results.push(ResultRecord {
            id: check.id.clone(),
            code,
            skill: check.skill.clone(),
            rerun,
            consecutive_failures,
        });
        self.save()
    }
    pub fn finish(&mut self, context: &Context, origin: &Path, code: i32) -> Result<bool> {
        let fingerprint = self.record.fingerprint(context, origin)?;
        let revision = match self.exported_revision() {
            Some(revision) => Some(revision.to_owned()),
            None => content::head(origin)?,
        };
        let index_matches = self
            .record
            .index_fingerprint()
            .map(|saved| content::index(origin).map(|current| current == saved))
            .transpose()?;
        let stable = fingerprint == self.record.fingerprint
            && revision == self.record.revision
            && index_matches.unwrap_or(true);
        self.record.status = if stable {
            "completed"
        } else {
            "inputs-changed"
        }
        .into();
        self.record.code = Some(code);
        self.save()?;
        Ok(stable)
    }
    fn save(&self) -> Result<()> {
        let directory = self
            .path
            .parent()
            .ok_or_else(|| anyhow::anyhow!("runtime file has no parent"))?;
        fs::create_dir_all(directory)?;
        crate::shared::artifacts::json::save(&self.path, &self.record)
    }
}
fn read(path: &Path) -> Result<Option<Record>> {
    match fs::read(path) {
        Ok(bytes) => Ok(Some(serde_json::from_slice(&bytes)?)),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error.into()),
    }
}
pub fn resume(context: &Context) -> Value {
    match observed(context) {
        Ok(value) => value,
        Err(error) => json!({"status": "unavailable", "error": format!("{error:#}")}),
    }
}
fn observed(context: &Context) -> Result<Value> {
    let path = context
        .path(&context.config.paths.runtime)?
        .join("checks.json");
    let Some(record) = read(&path)? else {
        return Ok(json!({"status": "absent"}));
    };
    let revision = content::head(&context.root)?;
    let worktree = content::fingerprint(context, &context.root, content::Tree::Working)?;
    let revision_matches = revision.is_some() && revision == record.revision;
    let content_matches = worktree == record.fingerprint;
    let index_matches = record
        .index_fingerprint()
        .map(|saved| content::index(&context.root).map(|current| current == saved))
        .transpose()?;
    let unchanged = revision_matches && content_matches && index_matches.unwrap_or(true);
    let current = unchanged && record.status == "completed";
    let full_run = record.only.is_none() && !record.selective;
    let full_gate_passed = current && full_run && record.code == Some(0);
    Ok(
        json!({"last_run": record, "revision_matches": revision_matches, "worktree_matches": content_matches, "index_matches": index_matches,
        "current": current, "full_gate_passed": full_gate_passed}),
    )
}
pub fn report(context: &Context) -> Result<()> {
    let value = resume(context);
    console::stdout(format!(
        "Check evidence: {}",
        serde_json::to_string(&value)?
    ));
    let results = value.pointer("/last_run/results").and_then(Value::as_array);
    for result in results.into_iter().flatten() {
        let repeated = result["consecutive_failures"].as_u64().unwrap_or_default() >= 2;
        if repeated {
            console::stdout(format!(
                "Repeated check failure [{id}]: {attempts} attempts. Guidance presented: {skill}. RERUN: {rerun}. Compare actual failures and revise guidance if applying it did not help; this does not prove the skill was read or the root cause repeated.",
                id = result["id"].as_str().unwrap_or_default(),
                attempts = result["consecutive_failures"],
                skill = result["skill"].as_str().unwrap_or_default(),
                rerun = result["rerun"].as_str().unwrap_or_default()
            ));
        }
    }
    Ok(())
}
