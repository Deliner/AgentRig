use anyhow::{Context, Result};
use serde::Deserialize;
use std::path::Path;

pub struct Message {
    pub rule: String,
    pub path: String,
    pub line: Option<usize>,
    pub text: String,
}

const COMPILER_MESSAGE: &str = "compiler-message";

#[derive(Deserialize)]
struct Record {
    reason: String,
    message: Option<Compiled>,
}
impl Record {
    fn compiled(self) -> Option<Compiled> {
        let compiler = self.reason == COMPILER_MESSAGE;
        if compiler { self.message } else { None }
    }
}
#[derive(Deserialize)]
struct Compiled {
    message: String,
    code: Option<Code>,
    level: String,
    spans: Vec<Span>,
}
#[derive(Deserialize)]
struct Code {
    code: String,
}
#[derive(Deserialize)]
struct Span {
    file_name: String,
    line_start: usize,
    is_primary: bool,
}

pub struct Locations<'a> {
    pub root: &'a Path,
    pub workspace: &'a Path,
    pub manifest: &'a Path,
}

impl Locations<'_> {
    pub fn messages(&self, output: &[u8]) -> Result<Vec<Message>> {
        let records = serde_json::Deserializer::from_slice(output).into_iter::<Record>();
        let mut messages = Vec::new();
        for record in records {
            let compiled = record.context("parse a Clippy message")?.compiled();
            messages.extend(compiled.and_then(|compiled| self.reported(compiled)));
        }
        Ok(messages)
    }
    fn reported(&self, compiled: Compiled) -> Option<Message> {
        let reportable = matches!(compiled.level.as_str(), "error" | "warning");
        let primary = compiled.spans.iter().find(|span| span.is_primary);
        let located = compiled.code.is_some() || primary.is_some();
        let path = primary.map_or_else(
            || self.manifest.to_path_buf(),
            |span| self.workspace.join(&span.file_name),
        );
        (reportable && located).then(|| Message {
            rule: compiled
                .code
                .map_or_else(|| "clippy".to_owned(), |code| code.code),
            path: path.trim_prefix(self.root).display().to_string(),
            line: primary.map(|span| span.line_start),
            text: compiled.message,
        })
    }
}
