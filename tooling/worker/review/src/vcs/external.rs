//! Process boundary for privately supplied VCS implementations.
use super::{Backend, Entry, FileKind, Observation, export};
use anyhow::{Context as _, Result, ensure};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    io::Seek,
    path::Path,
    process::{Command, Output},
};

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Adapter {
    pub command: Vec<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Reply<T> {
    version: u32,
    result: T,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct TreeEntry {
    path: String,
    kind: FileKind,
    object: String,
}

impl Adapter {
    pub fn validate(&self) -> Result<()> {
        ensure!(
            self.command.first().is_some_and(|value| !value.is_empty())
                && self.command.iter().all(|value| !value.contains('\0')),
            "external VCS command requires an executable and NUL-free arguments"
        );
        Ok(())
    }

    pub fn resolve(&self, root: &Path, reference: &str) -> Result<String> {
        revision(self.call(root, "resolve", json!({"reference": reference}))?)
    }

    pub fn head(&self, root: &Path) -> Result<Option<String>> {
        let value: Option<String> = self.call(root, "head", json!({}))?;
        value.map(revision).transpose()
    }

    pub fn observe(&self, root: &Path) -> Result<Observation> {
        let mut value: Observation = self.call(root, "observe", json!({}))?;
        let committed = !value.revision.is_empty();
        if committed {
            value.revision = revision(value.revision)?;
        }
        value.backend = Backend::External(self.clone());
        Ok(value)
    }

    pub fn parents(&self, root: &Path, id: &str) -> Result<Vec<String>> {
        let values: Vec<String> = self.call(root, "parents", json!({"revision": id}))?;
        values.into_iter().map(revision).collect()
    }

    pub fn tree(&self, root: &Path, id: &str) -> Result<BTreeMap<String, Entry>> {
        let entries: Vec<TreeEntry> = self.call(root, "tree", json!({"revision": id}))?;
        let mut tree = BTreeMap::new();
        for entry in entries {
            export::validate_path(&entry.path)?;
            let path = entry.path;
            let duplicate = tree
                .insert(
                    path.clone(),
                    Entry {
                        kind: entry.kind,
                        object: revision(entry.object)?,
                    },
                )
                .is_some();
            ensure!(!duplicate, "duplicate external VCS tree path: {path}");
        }
        Ok(tree)
    }

    pub fn read(&self, root: &Path, id: &str, path: &str) -> Result<Vec<u8>> {
        export::validate_path(path)?;
        self.call(root, "read", json!({"revision": id, "path": path}))
    }

    pub fn changed_paths(&self, root: &Path, revisions: (&str, &str)) -> Result<Vec<String>> {
        paths(self.call(root, "changed-paths", boundary(revisions))?)
    }

    pub fn working_files(&self, root: &Path) -> Result<Vec<String>> {
        paths(self.call(root, "working-files", json!({}))?)
    }

    pub fn diff(&self, root: &Path, revisions: (&str, &str)) -> Result<String> {
        self.call(root, "diff", boundary(revisions))
    }

    fn call<T: DeserializeOwned>(
        &self,
        root: &Path,
        operation: &str,
        arguments: Value,
    ) -> Result<T> {
        self.validate()?;
        let mut input = tempfile::tempfile()?;
        serde_json::to_writer(
            &input,
            &json!({"version": 1, "operation": operation, "arguments": arguments}),
        )?;
        input.rewind()?;
        let output = Command::new("bwrap")
            .args([
                "--die-with-parent",
                "--ro-bind",
                "/",
                "/",
                "--dev",
                "/dev",
                "--proc",
                "/proc",
                "--",
            ])
            .args(&self.command)
            .current_dir(root)
            .stdin(input)
            .output()
            .context("launch read-only external VCS adapter with bubblewrap")?;
        decode(operation, output)
    }
}

fn decode<T: DeserializeOwned>(operation: &str, output: Output) -> Result<T> {
    let unsupported = output.status.code() == Some(64);
    ensure!(
        !unsupported,
        "external VCS does not support {operation}: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    ensure!(
        output.status.success(),
        "external VCS {operation} failed ({}): {}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );
    let reply: Reply<T> = serde_json::from_slice(&output.stdout)
        .with_context(|| format!("invalid external VCS {operation} reply"))?;
    ensure!(
        reply.version == 1,
        "unsupported external VCS protocol version {}",
        reply.version
    );
    Ok(reply.result)
}

fn revision(value: String) -> Result<String> {
    ensure!(
        !value.trim().is_empty() && !value.chars().any(char::is_control),
        "external VCS must return one nonempty revision/object ID without control characters"
    );
    Ok(value)
}

fn paths(mut values: Vec<String>) -> Result<Vec<String>> {
    for value in &values {
        export::validate_path(value)?;
    }
    values.sort();
    values.dedup();
    Ok(values)
}

fn boundary((base, candidate): (&str, &str)) -> Value {
    json!({"base": base, "candidate": candidate})
}
