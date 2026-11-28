use super::messages::{Locations, Message};
use super::profile::denials;
use super::workspace;
use anyhow::{Context, Result, ensure};
use std::path::Path;
use std::process::Output;

const TARGETS: [&str; 4] = ["--bins", "--examples", "--message-format=json", "--"];

#[derive(Clone, Copy)]
pub struct Clippy<'a> {
    pub manifest: &'a Path,
    pub launch: &'a dyn Fn(&[String]) -> Result<Output>,
}

impl Clippy<'_> {
    pub fn messages(self, root: &Path) -> Vec<Message> {
        self.checked(root).unwrap_or_else(|error| {
            vec![Message {
                rule: "clippy".to_owned(),
                path: self.manifest.trim_prefix(root).display().to_string(),
                line: None,
                text: format!("Clippy could not check the project: {error:#}"),
            }]
        })
    }
    fn checked(self, root: &Path) -> Result<Vec<Message>> {
        let manifest = self
            .manifest
            .to_str()
            .context("the Cargo manifest path is not UTF-8")?;
        let inspect = ["cargo", "metadata", "--no-deps", "--format-version", "1"];
        let inspect = inspect.into_iter().chain(["--manifest-path", manifest]);
        let metadata = (self.launch)(&inspect.map(String::from).collect::<Vec<_>>())?;
        ensure!(
            metadata.status.success(),
            "cargo metadata failed: {}",
            stderr(&metadata)
        );
        let workspace = workspace::read(&metadata.stdout)?;
        let library = workspace.library.then_some("--lib");
        let cargo = ["cargo", "clippy", "--manifest-path", manifest].into_iter();
        let selected = cargo.chain(library).chain(TARGETS).map(String::from);
        let output = (self.launch)(&selected.chain(denials()).collect::<Vec<_>>())?;
        let locations = Locations {
            root,
            workspace: &workspace.root,
            manifest: self.manifest,
        };
        let found = locations.messages(&output.stdout)?;
        let failed = !output.status.success() && found.is_empty();
        ensure!(!failed, "cargo clippy failed: {}", stderr(&output));
        Ok(found)
    }
}

fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).trim().to_owned()
}
