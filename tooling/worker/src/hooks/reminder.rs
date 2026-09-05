use super::transcript;
use crate::util::{object, text};
use anyhow::Result;
use fs2::FileExt;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    env,
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
fn state_path(event: &Value, directory: Option<&Path>) -> Result<(File, PathBuf)> {
    let root = if let Some(path) = directory {
        path.to_owned()
    } else if let Ok(value) = env::var("COMPLEXITY_DISCIPLINE_STATE_DIR") {
        PathBuf::from(value)
    } else {
        let home = env::var("HOME")?;
        let base = env::var("XDG_STATE_HOME").unwrap_or(format!("{home}/.local/state"));
        let cwd = if text(event, "cwd").is_empty() {
            env::current_dir()?.to_string_lossy().into_owned()
        } else {
            text(event, "cwd").into()
        };
        PathBuf::from(base)
            .join("codex/complexity-discipline")
            .join(hash(&cwd))
    };
    fs::create_dir_all(&root)?;
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
pub fn start(event: &Value) -> Result<()> {
    start_at(event, None)
}
pub fn start_at(event: &Value, directory: Option<&Path>) -> Result<()> {
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
pub fn before(root: &Path, event: &Value) -> Result<Option<String>> {
    let config = object(&root.join(".agents/skills/complexity-discipline/context-reminder.json"));
    before_at(event, &config, None)
}
pub fn before_at(
    event: &Value,
    config: &Value,
    directory: Option<&Path>,
) -> Result<Option<String>> {
    let transcript = text(event, "transcript_path");
    if transcript.is_empty() {
        return Ok(None);
    }
    let mut attention = config["attention_interval_tokens"]
        .as_u64()
        .filter(|n| *n > 0)
        .unwrap_or(35_000);
    let mut full = config["full_refresh_interval_tokens"]
        .as_u64()
        .filter(|n| *n > attention)
        .unwrap_or(140_000);
    if full <= attention {
        attention = 35_000;
        full = 140_000;
    }
    let message = config["attention_message"]
        .as_str()
        .filter(|s| !s.trim().is_empty())
        .unwrap_or(ATTENTION);
    let (_lock, path) = state_path(event, directory)?;
    let mut state = object(&path);
    if text(&state, "transcript_path") != transcript {
        state = json!({"transcript_path": transcript, "scan_offset": 0,
            "baseline_tokens": 0, "allow_next_edit": false, "next_attention_tokens": attention});
    }
    let mut baseline = state["baseline_tokens"].as_u64().unwrap_or(0);
    let mut next = state["next_attention_tokens"].as_u64().unwrap_or(attention);
    if next == 0
        || !next.is_multiple_of(attention)
        || next > full.div_ceil(attention).saturating_mul(attention)
    {
        next = attention;
    }
    let mut allow = state["allow_next_edit"].as_bool().unwrap_or(false);
    let mut latest = state["latest_tokens"].as_u64();
    let (tokens, offset) = transcript::tokens(
        Path::new(transcript),
        state["scan_offset"].as_u64().unwrap_or(0),
    )?;
    for current in tokens {
        if latest.is_some_and(|prior| current < prior) {
            baseline = current;
            allow = false;
            next = attention;
        }
        latest = Some(current);
    }
    let mut reminder = None;
    if let Some(current) = latest {
        if baseline > current {
            baseline = current;
            next = attention;
            allow = false;
        }
        if allow {
            allow = false;
        } else {
            let delta = current - baseline;
            if delta >= full {
                baseline = current;
                next = attention;
                allow = true;
                reminder = Some(FULL.to_owned());
            } else if delta >= next {
                next = (delta / attention + 1).saturating_mul(attention);
                allow = true;
                reminder = Some(message.into());
            }
        }
        state["latest_tokens"] = json!(current);
    } else {
        state.as_object_mut().unwrap().remove("latest_tokens");
    }
    state["scan_offset"] = json!(offset);
    state["baseline_tokens"] = json!(baseline);
    state["next_attention_tokens"] = json!(next);
    state["allow_next_edit"] = json!(allow);
    save(&path, &state)?;
    Ok(reminder)
}
