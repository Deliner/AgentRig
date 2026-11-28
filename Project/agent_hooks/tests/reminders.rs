// DECISION: D019
// DECISION: D006
use super::{initialized, respond};
use crate::agent_hooks::dispatch;
use crate::shared::calling_agent::Agent;
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

fn skill(agent: Agent, name: &str) -> String {
    format!("{}/{name}/SKILL.md", agent.skills())
}

fn answer(root: &Path, event: &Value, agent: Agent) -> Value {
    dispatch(root, event, agent).unwrap().unwrap_or(Value::Null)
}

fn session() -> (TempDir, PathBuf) {
    let (temp, root) = initialized(&["--skills", "guides"]);
    for (name, before, after) in [
        (
            "project",
            "runtime: .cache/runtime",
            "runtime: .scratch/state",
        ),
        (
            "hooks",
            "reminder: guides/complexity-discipline/context-reminder.json",
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

fn usage(count: u64, agent: Agent) -> Value {
    match agent {
        Agent::ClaudeCode => json!({"type": "assistant", "message": {"usage": {
            "input_tokens": count / 2, "cache_creation_input_tokens": count / 4,
            "cache_read_input_tokens": count - count / 2 - count / 4, "output_tokens": 10000}}}),
        Agent::Codex => json!({"type": "event_msg", "payload": {"type": "token_count",
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
}

fn follow_schedule(root: &Path, edit: &Value, agent: Agent) {
    let transcript = root.join(".scratch/transcript.jsonl");
    for (index, (count, expected)) in SCHEDULE.into_iter().enumerate() {
        append(&transcript, &format!("{}\n", usage(count, agent)));
        let output = answer(root, edit, agent);
        reminded(&output, expected);
        let delivered = [0, CONTEXT_RESET].contains(&index);
        let guided = output.to_string().contains(&skill(agent, "edit-state"));
        assert_eq!(guided, delivered, "{index}: {output}");
    }
    let size = fs::metadata(&transcript).unwrap().len();
    assert_eq!(
        saved(&root.join(".scratch/state/reminders"))["scan_offset"],
        size
    );
}

// DECISION: D243
// INVARIANT: I007
#[test]
fn portable_reminder_schedule_retry_and_compaction() {
    for agent in Agent::ALL {
        let (_temp, root) = session();
        let common = common(&root);
        let start = merged(&common, json!({"hook_event_name": "SessionStart"}));
        let started = answer(&root, &start, agent).to_string();
        let discipline = skill(agent, "complexity-discipline");
        for expected in [FULL_REFRESH, &discipline, "Ledger/State.md"] {
            assert!(started.contains(expected), "{expected}: {started}");
        }
        let edit = writing(&common, "Ledger/State.md");
        follow_schedule(&root, &edit, agent);
        let compact = merged(&start, json!({"source": "compact"}));
        let refreshed = answer(&root, &compact, agent).to_string();
        assert!(refreshed.contains(FULL_REFRESH), "{refreshed}");
        let state = saved(&root.join(".scratch/state/reminders"));
        assert_eq!(state["baseline_tokens"], 150);
        let output = answer(&root, &edit, agent);
        let decided = output["hookSpecificOutput"].get("permissionDecision");
        assert!(decided.is_none(), "{output}");
        let guided = output.to_string().contains(&skill(agent, "edit-state"));
        assert!(guided, "{output}");
        let memory = [
            ("Plan/RIG-7.md", "edit-plan"),
            ("Decisions.md", "edit-decisions"),
            ("Invariants.md", "edit-invariants"),
        ];
        for (file, name) in memory {
            let edit = writing(&common, &format!("Ledger/{file}"));
            let output = answer(&root, &edit, agent).to_string();
            let guidance = skill(agent, name);
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
    for path in ["src/example.rs", "Ledger/State.md"] {
        let (_temp, root) = session();
        let runtime = root.join(".scratch/state");
        fs::write(&runtime, "occupied").unwrap();
        let tokens = format!("{}\n", usage(300, Agent::Codex));
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
        let guided = output
            .to_string()
            .contains(&skill(Agent::Codex, "edit-state"));
        assert_eq!(guided, path == "Ledger/State.md", "{path}: {output}");
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
    fs::write(
        root.join(".agentrig/skills/complexity-discipline/context-reminder.json"),
        settings,
    )
    .unwrap();
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
    let reminders = root.join(".cache/runtime/reminders");
    let record = usage(100, Agent::Codex).to_string();
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
