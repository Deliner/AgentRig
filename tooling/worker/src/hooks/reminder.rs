// DECISION: D006
use super::transcript;
use crate::util::{object, text};
use anyhow::{Context, Result};
use fs2::FileExt;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    fs::{self, File, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
};

// DECISION: D015
pub const FULL: &str = include_str!("full-refresh.txt");
const ATTENTION: &str = include_str!("attention.txt");
fn hash(value: &str) -> String {
    format!("{:x}", Sha256::digest(value.as_bytes()))
}
fn state_path(event: &Value, directory: &Path) -> Result<(File, PathBuf)> {
    let root = directory;
    fs::create_dir_all(root)?;
    let session = ["session_id", "transcript_path"]
        .iter()
        .map(|key| text(event, key))
        .find(|value| !value.is_empty())
        .unwrap_or("session");
    let stem = hash(session);
    let lock = OpenOptions::new()
        .create(true)
        .append(true)
        .open(root.join(format!("{stem}.lock")))?;
    lock.lock_exclusive()?;
    Ok((lock, root.join(format!("{stem}.json"))))
}
fn save(path: &Path, state: &Value) -> Result<()> {
    let mut temporary = tempfile::NamedTempFile::new_in(path.parent().unwrap())?;
    serde_json::to_writer(&mut temporary, state)?;
    writeln!(temporary)?;
    temporary.persist(path)?;
    Ok(())
}
pub fn start_at(event: &Value, directory: &Path) -> Result<()> {
    let transcript = text(event, "transcript_path");
    let (tokens, offset) = transcript::tokens(Path::new(transcript), 0).unwrap_or_default();
    let baseline = tokens.last().copied().unwrap_or(0);
    let (_lock, path) = state_path(event, directory)?;
    save(
        &path,
        &json!({"transcript_path": transcript, "scan_offset": offset,
        "baseline_tokens": baseline, "latest_tokens": baseline, "allow_next_edit": false}),
    )
}
pub fn before_at(event: &Value, config: &Value, directory: &Path) -> Result<Option<String>> {
    let transcript = text(event, "transcript_path");
    let missing_transcript = transcript.is_empty();
    if missing_transcript {
        return Ok(None);
    }
    let attention = config["attention_interval_tokens"]
        .as_u64()
        .context("attention interval missing")?;
    let full = config["full_refresh_interval_tokens"]
        .as_u64()
        .context("full refresh interval missing")?;
    let message = config["attention_message"]
        .as_str()
        .filter(|s| !s.trim().is_empty())
        .unwrap_or(ATTENTION);
    let (_lock, path) = state_path(event, directory)?;
    let mut state = object(&path);
    let changed_transcript = text(&state, "transcript_path") != transcript;
    if changed_transcript {
        state = json!({"transcript_path": transcript, "scan_offset": 0,
            "baseline_tokens": 0, "allow_next_edit": false, "next_attention_tokens": attention});
    }
    let mut progress = Progress::load(&state, attention, full);
    let (tokens, offset) = transcript::tokens(
        Path::new(transcript),
        state["scan_offset"].as_u64().unwrap_or(0),
    )?;
    for current in tokens {
        progress.observe(current, attention);
    }
    let reminder = progress.reminder(attention, full, message);
    progress.store(&mut state);
    state["scan_offset"] = json!(offset);
    save(&path, &state)?;
    Ok(reminder)
}
struct Progress {
    baseline: u64,
    next: u64,
    allow: bool,
    latest: Option<u64>,
}
impl Progress {
    fn load(state: &Value, attention: u64, full: u64) -> Self {
        let mut next = state["next_attention_tokens"].as_u64().unwrap_or(attention);
        let invalid_schedule = next == 0
            || !next.is_multiple_of(attention)
            || next > full.div_ceil(attention).saturating_mul(attention);
        if invalid_schedule {
            next = attention;
        }
        Self {
            baseline: state["baseline_tokens"].as_u64().unwrap_or(0),
            next,
            allow: state["allow_next_edit"].as_bool().unwrap_or(false),
            latest: state["latest_tokens"].as_u64(),
        }
    }
    fn observe(&mut self, current: u64, attention: u64) {
        let context_reset = self.latest.is_some_and(|prior| current < prior);
        if context_reset {
            self.reset(current, attention);
        }
        self.latest = Some(current);
    }
    fn reset(&mut self, current: u64, attention: u64) {
        self.baseline = current;
        self.next = attention;
        self.allow = false;
    }
    fn reminder(&mut self, attention: u64, full: u64, message: &str) -> Option<String> {
        let current = self.latest?;
        let invalid_baseline = self.baseline > current;
        if invalid_baseline {
            self.reset(current, attention);
        }
        if self.allow {
            self.allow = false;
            return None;
        }
        let delta = current - self.baseline;
        let refresh_due = delta >= full;
        if refresh_due {
            self.reset(current, attention);
            self.allow = true;
            return Some(FULL.to_owned());
        }
        let attention_due = delta >= self.next;
        if attention_due {
            self.next = (delta / attention + 1).saturating_mul(attention);
            self.allow = true;
            return Some(message.into());
        }
        None
    }
    fn store(&self, state: &mut Value) {
        if let Some(current) = self.latest {
            state["latest_tokens"] = json!(current);
        } else {
            state.as_object_mut().unwrap().remove("latest_tokens");
        }
        state["baseline_tokens"] = json!(self.baseline);
        state["next_attention_tokens"] = json!(self.next);
        state["allow_next_edit"] = json!(self.allow);
    }
}
