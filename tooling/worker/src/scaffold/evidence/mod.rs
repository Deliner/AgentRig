// DECISION: D022
mod content;
pub use content::native_index;

use super::config::{self, Check, Context};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{
    fs,
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
    revision_export: bool,
    fingerprint: String,
    staged: bool,
    index_fingerprint: Option<String>,
    only: Option<String>,
    status: String,
    code: Option<i32>,
    results: Vec<ResultRecord>,
}
pub enum Input {
    Worktree,
    Index(String),
    Revision(String),
}

impl Record {
    fn exported_revision(&self) -> Option<&str> {
        self.revision.as_deref().filter(|_| self.revision_export)
    }

    fn fingerprint(&self, context: &Context, origin: &Path) -> Result<String> {
        match self.exported_revision() {
            Some(revision) => content::revision_fingerprint(context, origin, revision),
            None => content::fingerprint(context, origin, self.staged),
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
        let (index_fingerprint, selected) = match input {
            Input::Worktree => (None, None),
            Input::Index(index) => (Some(index), None),
            Input::Revision(revision) => (None, Some(revision)),
        };
        let directory = config::relative(origin, &context.config.paths.runtime)?;
        let path = directory.join("checks.json");
        let mut record = Record {
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs(),
            revision_export: selected.is_some(),
            revision: match selected {
                Some(revision) => Some(revision),
                None => content::head(context, origin)?,
            },
            fingerprint: String::new(),
            staged: index_fingerprint.is_some(),
            index_fingerprint,
            only: only.map(str::to_owned),
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
    pub fn rerun(&self, root: &Path, id: &str) -> String {
        let mut args = vec!["check".to_owned(), "--only".into(), id.into()];
        if self.record.staged {
            args.push("--staged".into());
        }
        if let Some(revision) = self.exported_revision() {
            args.extend(["--revision".into(), revision.into()]);
        }
        crate::diagnostics::rerun(root, &args)
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
            None => content::head(context, origin)?,
        };
        let index_matches = self
            .record
            .index_fingerprint
            .as_ref()
            .map(|saved| {
                content::index(&context.config.vcs.backend, origin).map(|current| current == *saved)
            })
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
        let directory = self.path.parent().expect("runtime file parent");
        fs::create_dir_all(directory)?;
        let mut file = tempfile::NamedTempFile::new_in(directory)?;
        serde_json::to_writer(file.as_file_mut(), &self.record)?;
        file.as_file().sync_all()?;
        file.persist(&self.path)?;
        Ok(())
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
    let revision = content::head(context, &context.root)?;
    let worktree = content::fingerprint(context, &context.root, false)?;
    let revision_matches = revision.is_some() && revision == record.revision;
    let content_matches = worktree == record.fingerprint;
    let index_matches = record
        .index_fingerprint
        .as_ref()
        .map(|saved| {
            content::index(&context.config.vcs.backend, &context.root)
                .map(|current| current == *saved)
        })
        .transpose()?;
    let current = revision_matches
        && content_matches
        && index_matches.unwrap_or(true)
        && record.status == "completed";
    let full_gate_passed = current && record.only.is_none() && record.code == Some(0);
    Ok(
        json!({"last_run": record, "revision_matches": revision_matches, "worktree_matches": content_matches, "index_matches": index_matches,
        "current": current, "full_gate_passed": full_gate_passed}),
    )
}
pub fn report(context: &Context) -> Result<()> {
    let value = resume(context);
    println!("Check evidence: {}", serde_json::to_string(&value)?);
    let results = value.pointer("/last_run/results").and_then(Value::as_array);
    for result in results.into_iter().flatten() {
        let repeated = result["consecutive_failures"].as_u64().unwrap_or_default() >= 2;
        if repeated {
            println!(
                "Repeated check failure [{}]: {} attempts. Guidance presented: {}. RERUN: {}. Compare actual failures and revise guidance if applying it did not help; this does not prove the skill was read or the root cause repeated.",
                result["id"].as_str().unwrap_or_default(),
                result["consecutive_failures"],
                result["skill"].as_str().unwrap_or_default(),
                result["rerun"].as_str().unwrap_or_default()
            );
        }
    }
    Ok(())
}
