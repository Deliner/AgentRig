// DECISION: D006
use super::input::{object, text};
use super::transcript;
use crate::shared::fs::{self, File, OpenOptions};
use anyhow::{Context, Result};
use fs2::FileExt;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
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
    let lock = fs::open_with(
        OpenOptions::new().create(true).append(true),
        root.join(format!("{stem}.lock")),
    )?;
    lock.lock_exclusive()?;
    Ok((lock, root.join(format!("{stem}.json"))))
}
fn save(path: &Path, state: &Value) -> Result<()> {
    let parent = path
        .parent()
        .ok_or_else(|| anyhow::anyhow!("reminder file has no parent"))?;
    let mut temporary = tempfile::NamedTempFile::new_in(parent)?;
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
pub struct Session {
    _lock: File,
    path: PathBuf,
    state: Value,
}
// DECISION: D129
impl Session {
    pub fn state_mut(&mut self) -> &mut Value {
        &mut self.state
    }
    pub fn save(&self) -> Result<()> {
        save(&self.path, &self.state)
    }
}
// DECISION: D047
pub fn open(event: &Value, directory: &Path) -> Result<Session> {
    let (lock, path) = state_path(event, directory)?;
    let state = object(&path);
    Ok(Session {
        _lock: lock,
        path,
        state,
    })
}
pub struct Schedule {
    attention: u64,
    full: u64,
    message: String,
}
impl Schedule {
    pub fn read(config: &Value) -> Result<Self> {
        Ok(Self {
            attention: config["attention_interval_tokens"]
                .as_u64()
                .context("attention interval missing")?,
            full: config["full_refresh_interval_tokens"]
                .as_u64()
                .context("full refresh interval missing")?,
            message: config["attention_message"]
                .as_str()
                .filter(|s| !s.trim().is_empty())
                .unwrap_or(ATTENTION)
                .to_owned(),
        })
    }
    pub fn due(&self, event: &Value, state: &mut Value) -> Result<Option<String>> {
        let transcript = text(event, "transcript_path");
        let missing_transcript = transcript.is_empty();
        if missing_transcript {
            return Ok(None);
        }
        let changed_transcript = text(state, "transcript_path") != transcript;
        if changed_transcript {
            *state = json!({"transcript_path": transcript, "scan_offset": 0,
                "baseline_tokens": 0, "allow_next_edit": false, "next_attention_tokens": self.attention});
        }
        let mut progress = Progress::load(state, self);
        let (tokens, offset) = transcript::tokens(
            Path::new(transcript),
            state["scan_offset"].as_u64().unwrap_or(0),
        )?;
        let context_reset = progress.advance(tokens, self);
        let reminder = progress.reminder(self);
        progress.store(state);
        if context_reset {
            forget_delivered(state);
        }
        state["scan_offset"] = json!(offset);
        Ok(reminder)
    }
}
fn forget_delivered(state: &mut Value) {
    let Some(object) = state.as_object_mut() else {
        return;
    };
    object.remove("delivered");
}
struct Progress {
    baseline: u64,
    next: u64,
    allow: bool,
    latest: Option<u64>,
}
impl Progress {
    fn load(state: &Value, schedule: &Schedule) -> Self {
        let attention = schedule.attention;
        let mut next = state["next_attention_tokens"].as_u64().unwrap_or(attention);
        let invalid_schedule = next == 0
            || !next.is_multiple_of(attention)
            || next > schedule.full.div_ceil(attention).saturating_mul(attention);
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
    fn advance(&mut self, tokens: Vec<u64>, schedule: &Schedule) -> bool {
        let mut context_reset = false;
        for current in tokens {
            context_reset |= self.observe(current, schedule);
        }
        let repaired = self.repair_baseline(schedule);
        context_reset || repaired
    }
    fn observe(&mut self, current: u64, schedule: &Schedule) -> bool {
        let context_reset = self.latest.is_some_and(|prior| current < prior);
        if context_reset {
            self.reset(current, schedule);
        }
        self.latest = Some(current);
        context_reset
    }
    fn repair_baseline(&mut self, schedule: &Schedule) -> bool {
        let Some(current) = self.latest else {
            return false;
        };
        let invalid_baseline = self.baseline > current;
        if invalid_baseline {
            self.reset(current, schedule);
        }
        invalid_baseline
    }
    fn reset(&mut self, current: u64, schedule: &Schedule) {
        self.baseline = current;
        self.next = schedule.attention;
        self.allow = false;
    }
    fn reminder(&mut self, schedule: &Schedule) -> Option<String> {
        let current = self.latest?;
        if self.allow {
            self.allow = false;
            return None;
        }
        let delta = current - self.baseline;
        let refresh_due = delta >= schedule.full;
        if refresh_due {
            self.reset(current, schedule);
            self.allow = true;
            return Some(FULL.to_owned());
        }
        let attention_due = delta >= self.next;
        if attention_due {
            self.next = (delta / schedule.attention + 1).saturating_mul(schedule.attention);
            self.allow = true;
            return Some(schedule.message.clone());
        }
        None
    }
    fn store(&self, state: &mut Value) {
        let Some(object) = state.as_object_mut() else {
            return;
        };
        if let Some(current) = self.latest {
            object.insert("latest_tokens".into(), json!(current));
        } else {
            object.remove("latest_tokens");
        }
        object.insert("baseline_tokens".into(), json!(self.baseline));
        object.insert("next_attention_tokens".into(), json!(self.next));
        object.insert("allow_next_edit".into(), json!(self.allow));
    }
}
