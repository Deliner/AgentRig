use super::{
    model::{Journal, Plan},
    review, storage,
};
use crate::scaffold::receipt as manifest;
use anyhow::{Context as _, Result, ensure};
use std::{
    fs,
    path::{Path, PathBuf},
};

pub struct Operation {
    pub directory: PathBuf,
    pub plan: Plan,
    pub journal: Journal,
}
impl Operation {
    pub fn open(root: &Path) -> Result<Self> {
        let directory = storage::directory(root)?.join("operation");
        let journal: Journal = serde_json::from_slice(&fs::read(directory.join("journal.json"))?)?;
        ensure!(journal.version == 1, "unsupported upgrade journal");
        let bytes = fs::read(directory.join("plan.json"))?;
        ensure!(
            manifest::checksum(&bytes) == journal.plan_sha256,
            "saved upgrade plan changed"
        );
        let plan: Plan = serde_json::from_slice(&bytes)?;
        ensure!(
            Path::new(&plan.project) == root,
            "operation belongs to another project"
        );
        Ok(Self {
            directory,
            plan,
            journal,
        })
    }
    pub fn prepare(root: &Path, path: &Path) -> Result<Self> {
        let path = path.canonicalize()?;
        let directory = storage::directory(root)?;
        let operation = directory.join("operation");
        let exists = operation.exists();
        if exists {
            let current = Self::open(root)?;
            let archive_previous = current.journal.phase == "rolled-back"
                || (current.journal.phase == "applied"
                    && current.journal.plan != path.to_string_lossy());
            if archive_previous {
                let plan = review::load(root, &path)?;
                let archive = tempfile::Builder::new()
                    .prefix("restored-")
                    .tempdir_in(&directory)?;
                fs::rename(&current.directory, archive.path().join("operation"))?;
                let _ = archive.keep();
                fs::File::open(&directory)?.sync_all()?;
                return Self::persist(&directory, &path, plan);
            } else {
                ensure!(
                    current.journal.plan == path.to_string_lossy(),
                    "another upgrade operation exists; use its plan or rollback"
                );
                return Ok(current);
            }
        }
        let plan = review::load(root, &path)?;
        Self::persist(&directory, &path, plan)
    }
    fn persist(directory: &Path, path: &Path, plan: Plan) -> Result<Self> {
        let operation = directory.join("operation");
        let pending = tempfile::Builder::new()
            .prefix("operation-")
            .tempdir_in(directory)?;
        copy_payloads(
            &plan,
            path.parent().context("plan directory required")?,
            pending.path(),
        )?;
        let bytes = serde_json::to_vec_pretty(&plan)?;
        storage::atomic(&pending.path().join("plan.json"), &bytes, 0o600)?;
        let journal = Journal {
            version: 1,
            plan: path.to_string_lossy().into_owned(),
            plan_sha256: manifest::checksum(&bytes),
            phase: "applying".into(),
            completed: vec![],
            restored: vec![],
            checks: Default::default(),
            next_action: format!("upgrade apply {} or upgrade rollback", path.display()),
        };
        storage::json(&pending.path().join("journal.json"), &journal)?;
        fs::rename(pending.path(), &operation)?;
        fs::File::open(directory)?.sync_all()?;
        Ok(Self {
            directory: operation,
            plan,
            journal,
        })
    }
    pub fn save(&self) -> Result<()> {
        storage::json(&self.directory.join("journal.json"), &self.journal)
    }
    pub fn phase(&mut self, phase: &str) -> Result<()> {
        self.journal.phase = phase.into();
        self.save()
    }
}
fn copy_payloads(plan: &Plan, source: &Path, target: &Path) -> Result<()> {
    for change in plan.files.values() {
        for state in [&change.before, &change.after] {
            if let Some(hash) = &state.sha256 {
                storage::blob(target, &storage::payload(source, hash)?)?;
            }
        }
    }
    Ok(())
}
