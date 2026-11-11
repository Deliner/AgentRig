// DECISION: D019
// DECISION: D006
use super::{initialized, respond};
use crate::shared::testing::{FAIL, GATE, project};
use serde_json::{Value, json};
use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
};
use tempfile::TempDir;

const SCHEDULE: [(u64, Option<&str>); 10] = [
    (99, None),
    (100, Some("REASSESS_CURRENT_WORK")),
    (100, None),
    (200, Some("REASSESS_CURRENT_WORK")),
    (201, None),
    (300, Some("FULL_REFRESH_REQUIRED")),
    (301, None),
    (50, None),
    (149, None),
    (150, Some("REASSESS_CURRENT_WORK")),
];
const CONTEXT_RESET: usize = 7;
const FULL_REFRESH: &str = "FULL_REFRESH_REQUIRED";
const UNAVAILABLE: &str = "Hook session state unavailable";
const STATE_GUIDANCE: &str = "guides/edit-state/SKILL.md";

fn session(frontend: &str) -> (TempDir, PathBuf) {
    let options = [
        "--memory",
        "notes",
        "--skills",
        "guides",
        "--frontend",
        frontend,
    ];
    let (temp, root) = initialized(&options);
    for (name, before, after) in [
        (
            "project",
            "runtime: .agentrig/runtime",
            "runtime: .scratch/state",
        ),
        (
            "hooks",
            "reminder: .agentrig/reminder.json",
            "reminder: .scratch/reminder.json",
        ),
    ] {
        let path = root.join(format!(".agentrig/{name}.yaml"));
        let text = fs::read_to_string(&path).unwrap();
        fs::write(&path, text.replace(before, after)).unwrap();
    }
    let scratch = root.join(".scratch");
    fs::create_dir(&scratch).unwrap();
    let settings = json!({"attention_interval_tokens": 100, "full_refresh_interval_tokens": 300,
        "attention_message": "REASSESS_CURRENT_WORK"});
    fs::write(scratch.join("reminder.json"), settings.to_string()).unwrap();
    fs::write(scratch.join("transcript.jsonl"), "").unwrap();
    (temp, root)
}

fn common(root: &Path) -> Value {
    let transcript = root.join(".scratch/transcript.jsonl");
    json!({"cwd": root, "session_id": "portable", "transcript_path": transcript})
}

fn merged(common: &Value, fields: Value) -> Value {
    let mut event = common.clone();
    let fields = fields.as_object().unwrap().clone();
    event.as_object_mut().unwrap().extend(fields);
    event
}

fn writing(common: &Value, path: &str) -> Value {
    let fields = json!({"hook_event_name": "PreToolUse", "tool_name": "Write",
        "tool_input": {"file_path": path}});
    merged(common, fields)
}

fn usage(count: u64, frontend: &str) -> Value {
    match frontend {
        "claude-code" => json!({"type": "assistant", "message": {"usage": {
            "input_tokens": count / 2, "cache_creation_input_tokens": count / 4,
            "cache_read_input_tokens": count - count / 2 - count / 4, "output_tokens": 10000}}}),
        _ => json!({"type": "event_msg", "payload": {"type": "token_count",
            "info": {"last_token_usage": {"input_tokens": count}}}}),
    }
}

fn append(transcript: &Path, text: &str) {
    let mut file = OpenOptions::new().append(true).open(transcript).unwrap();
    file.write_all(text.as_bytes()).unwrap();
}

fn saved(directory: &Path) -> Value {
    let entries = fs::read_dir(directory)
        .unwrap()
        .map(|entry| entry.unwrap().path());
    let mut states = entries.filter(|path| path.extension().is_some_and(|kind| kind == "json"));
    serde_json::from_str(&fs::read_to_string(states.next().unwrap()).unwrap()).unwrap()
}

fn reminded(output: &Value, expected: Option<&str>) {
    let specific = &output["hookSpecificOutput"];
    match expected {
        None => assert!(specific.get("permissionDecision").is_none(), "{output}"),
        Some(text) => {
            assert_eq!(specific["permissionDecision"], "deny", "{output}");
            let reason = specific["permissionDecisionReason"].as_str().unwrap();
            assert!(reason.contains(text), "{text}: {reason}");
        }
    }
    assert!(!output.to_string().contains(".agents/"), "{output}");
}

fn follow_schedule(root: &Path, edit: &Value, frontend: &str) {
    let transcript = root.join(".scratch/transcript.jsonl");
    for (index, (count, expected)) in SCHEDULE.into_iter().enumerate() {
        append(&transcript, &format!("{}\n", usage(count, frontend)));
        let output = respond(root, edit);
        reminded(&output, expected);
        let delivered = [0, CONTEXT_RESET].contains(&index);
        let guided = output.to_string().contains(STATE_GUIDANCE);
        assert_eq!(guided, delivered, "{index}: {output}");
    }
    let size = fs::metadata(&transcript).unwrap().len();
    assert_eq!(
        saved(&root.join(".scratch/state/reminders"))["scan_offset"],
        size
    );
}

// INVARIANT: I007
#[test]
fn portable_reminder_schedule_retry_and_compaction() {
    for frontend in ["codex", "claude-code"] {
        let (_temp, root) = session(frontend);
        let common = common(&root);
        let start = merged(&common, json!({"hook_event_name": "SessionStart"}));
        let started = respond(&root, &start).to_string();
        let skill = "guides/complexity-discipline/SKILL.md";
        for expected in [FULL_REFRESH, skill, "notes/State.md"] {
            assert!(started.contains(expected), "{expected}: {started}");
        }
        for absent in ["Ledger/", ".agents/"] {
            assert!(!started.contains(absent), "{absent}: {started}");
        }
        let edit = writing(&common, "notes/State.md");
        follow_schedule(&root, &edit, frontend);
        let compact = merged(&start, json!({"source": "compact"}));
        let refreshed = respond(&root, &compact).to_string();
        assert!(refreshed.contains(FULL_REFRESH), "{refreshed}");
        let state = saved(&root.join(".scratch/state/reminders"));
        assert_eq!(state["baseline_tokens"], 150);
        let output = respond(&root, &edit);
        let decided = output["hookSpecificOutput"].get("permissionDecision");
        assert!(decided.is_none(), "{output}");
        assert!(output.to_string().contains(STATE_GUIDANCE), "{output}");
        for name in ["Plan", "Decisions", "Invariants"] {
            let edit = writing(&common, &format!("notes/{name}.md"));
            let output = respond(&root, &edit).to_string();
            let guidance = format!("guides/edit-{}/SKILL.md", name.to_lowercase());
            assert!(output.contains(&guidance), "{guidance}: {output}");
        }
    }
}

#[test]
fn no_configured_discipline_does_not_invent_a_skill() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().canonicalize().unwrap();
    project(&root, FAIL, GATE);
    fs::write(root.join(".runtime"), "occupied").unwrap();
    let output = respond(&root, &json!({"hook_event_name": "SessionStart"})).to_string();
    assert!(output.contains("notes/State.md"), "{output}");
    for absent in [".agents/", FULL_REFRESH, UNAVAILABLE] {
        assert!(!output.contains(absent), "{absent}: {output}");
    }
}

#[test]
fn reminder_storage_failure_is_visible_and_recovers() {
    for path in ["src/example.rs", "notes/State.md"] {
        let (_temp, root) = session("codex");
        let runtime = root.join(".scratch/state");
        fs::write(&runtime, "occupied").unwrap();
        let tokens = format!("{}\n", usage(300, "codex"));
        append(&root.join(".scratch/transcript.jsonl"), &tokens);
        let fields = json!({"hook_event_name": "PreToolUse", "tool_name": "Edit",
            "tool_input": {"file_path": path}});
        let event = merged(&common(&root), fields);
        let output = respond(&root, &event);
        let specific = &output["hookSpecificOutput"];
        assert!(specific.get("permissionDecision").is_none(), "{output}");
        let context = specific["additionalContext"].as_str().unwrap();
        let reminders = runtime.join("reminders");
        for expected in [UNAVAILABLE, reminders.to_str().unwrap()] {
            assert!(context.contains(expected), "{expected}: {context}");
        }
        let guided = output.to_string().contains(STATE_GUIDANCE);
        assert_eq!(guided, path == "notes/State.md", "{path}: {output}");
        fs::remove_file(&runtime).unwrap();
        let recovered = respond(&root, &event).to_string();
        assert!(recovered.contains(FULL_REFRESH), "{recovered}");
        assert!(!recovered.contains(UNAVAILABLE), "{recovered}");
    }
}

#[test]
fn partial_and_truncated_transcripts_recover() {
    let (_temp, root) = initialized(&[]);
    let settings = r#"{"attention_interval_tokens": 100, "full_refresh_interval_tokens": 300}"#;
    fs::write(root.join(".agentrig/reminder.json"), settings).unwrap();
    let transcript = root.join("transcript.jsonl");
    fs::write(&transcript, "").unwrap();
    let common = json!({"session_id": "partial", "transcript_path": transcript});
    respond(
        &root,
        &merged(&common, json!({"hook_event_name": "SessionStart"})),
    );
    let fields = json!({"hook_event_name": "PreToolUse", "tool_name": "Edit",
        "tool_input": {"path": "Ledger/State.md"}});
    let event = merged(&common, fields);
    let denied = || respond(&root, &event).to_string().contains("deny");
    let reminders = root.join(".agentrig/runtime/reminders");
    let record = usage(100, "codex").to_string();
    fs::write(&transcript, record.strip_suffix("}}").unwrap()).unwrap();
    assert!(!denied());
    assert_eq!(saved(&reminders)["scan_offset"], 0);
    fs::write(&transcript, format!("{record}\n")).unwrap();
    assert!(denied());
    assert!(!denied());
    fs::write(&transcript, format!("{}\n", record.replace("100", "0"))).unwrap();
    assert!(!denied());
    let state = saved(&reminders);
    assert_eq!(
        state["scan_offset"],
        fs::metadata(&transcript).unwrap().len()
    );
    assert_eq!(state["latest_tokens"], 0);
    append(&transcript, &format!("invalid complete record\n{record}\n"));
    assert!(denied());
}
