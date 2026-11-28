// DECISION: D013
// DECISION: D014
// DECISION: D015
use super::{initialized, pre_tool, project, respond};
use crate::shared::testing::{git, run, runtime, snapshot, write};
use serde_json::{Value, json};
use std::os::unix::fs::{MetadataExt, symlink};
use std::{fs, path::Path};

const LEDGER_EDITS: [(&str, &str, &[&str]); 20] = [
    ("Edit", r#"{"file_path": "Ledger/Plan.md"}"#, &["edit-plan"]),
    (
        "Edit",
        r#"{"file_path": "Ledger/Backlog.md"}"#,
        &["edit-backlog"],
    ),
    (
        "Write",
        r#"{"file_path": "Ledger/Backlog/001.md"}"#,
        &["edit-backlog"],
    ),
    ("Write", r#"{"path": "Ledger/State.md"}"#, &["edit-state"]),
    (
        "Edit",
        r#"{"file_path": "Ledger/Decisions.md"}"#,
        &["edit-decisions", "norm-or-choice"],
    ),
    (
        "Write",
        r#"{"file_path": "Ledger/Invariants.md"}"#,
        &["edit-invariants"],
    ),
    (
        "Edit",
        r#"{"file_path": "Ledger/Glossary.md"}"#,
        &["edit-glossary"],
    ),
    (
        "Edit",
        r#"{"file_path": "Ledger/Plan/001.md"}"#,
        &["edit-plan"],
    ),
    (
        "Edit",
        r#"{"file_path": "Ledger/Decisions/001.md"}"#,
        &["edit-decisions", "norm-or-choice"],
    ),
    (
        "Edit",
        r#"{"file_path": "Ledger/Invariants/001.md"}"#,
        &["edit-invariants"],
    ),
    ("Edit", r#"{"file_path": "docs/State.md"}"#, &[]),
    ("Edit", r#"{"file_path": "Ledger/Plan-other.md"}"#, &[]),
    (
        "Edit",
        r#"{"file_path": "Ledger/Plan/../State.md"}"#,
        &["edit-state"],
    ),
    (
        "Edit",
        r#"{"file_path": "{root}/Ledger/State.md"}"#,
        &["edit-state"],
    ),
    ("Edit", r#"{"file_path": "{outside}/Ledger/State.md"}"#, &[]),
    (
        "apply_patch",
        r#""*** Begin Patch\n*** Update File: Ledger/Plan.md\n*** Update File: Ledger/Plan/001.md\n*** Delete File: Ledger/State.md\n*** Add File: Ledger/Invariants/013.md\n*** End Patch""#,
        &["edit-plan", "edit-invariants", "edit-state"],
    ),
    (
        "apply_patch",
        r#"{"patch": "*** Update File: Ledger/Decisions/001.md\n*** Move to: Ledger/Invariants/001.md\n"}"#,
        &["edit-decisions", "edit-invariants", "norm-or-choice"],
    ),
    (
        "apply_patch",
        r#"{"input": "*** Add File: Ledger/State.md\n+# State\n"}"#,
        &["edit-state"],
    ),
    (
        "apply_patch",
        r#"{"command": "*** Begin Patch\n*** Update File: Ledger/Plan.md\n*** End Patch"}"#,
        &["edit-plan"],
    ),
    (
        "apply_patch",
        r#"{"patch_text": "*** Update File: docs/example.md\n+*** Update File: Ledger/Plan.md\n"}"#,
        &[],
    ),
];
const MEMORY_SKILLS: [&str; 7] = [
    "edit-plan",
    "edit-backlog",
    "edit-decisions",
    "edit-invariants",
    "edit-state",
    "edit-glossary",
    "norm-or-choice",
];

fn in_session(root: &Path, mut event: Value, session: &str) -> Value {
    event["session_id"] = json!(session);
    respond(root, &event)
}

fn guidance(root: &Path, output: &Value, expected: &[&str]) {
    let unguided = expected.is_empty();
    if unguided {
        assert!(output.is_null(), "{output}");
        return;
    }
    let specific = &output["hookSpecificOutput"];
    let decision = specific["permissionDecision"].as_str();
    // DECISION: D186
    let decision_record = expected.contains(&"norm-or-choice");
    let message = if decision_record {
        assert_eq!(decision, Some("deny"), "{output}");
        specific["permissionDecisionReason"].as_str().unwrap()
    } else {
        assert_eq!(decision, None, "{output}");
        specific["additionalContext"].as_str().unwrap()
    };
    for skill in MEMORY_SKILLS {
        let path = root.join(".agents/skills").join(skill).join("SKILL.md");
        assert!(path.is_file(), "{}", path.display());
        let count = message.matches(path.to_str().unwrap()).count();
        let delivered = usize::from(expected.contains(&skill));
        assert_eq!(count, delivered, "{skill}: {message}");
    }
}

// INVARIANT: I012
#[test]
fn ledger_edit_guidance() {
    let (_temp, root) = initialized(&["--skills", "guides"]);
    let outside = root.parent().unwrap().to_str().unwrap();
    for (index, (tool, payload, expected)) in LEDGER_EDITS.into_iter().enumerate() {
        let payload = payload.replace("{root}", root.to_str().unwrap());
        let payload = serde_json::from_str(&payload.replace("{outside}", outside)).unwrap();
        let output = in_session(
            &root,
            pre_tool(&root, tool, payload),
            &format!("edit-{index}"),
        );
        guidance(&root, &output, expected);
    }
}

#[test]
fn archived_plan_edits_are_guided_without_writing() {
    let (_temp, root) = initialized(&["--skills", "guides"]);
    let archive = root.join("Ledger/Archive");
    fs::create_dir_all(archive.join("Plan")).unwrap();
    let paths = [archive.join("Plan.md"), archive.join("Plan/001.md")];
    for path in &paths {
        fs::write(path, "# Archived\n").unwrap();
    }
    let before = snapshot(&archive);
    for (index, path) in paths.iter().enumerate() {
        let event = pre_tool(&root, "Write", json!({"file_path": path}));
        let output = in_session(&root, event, &format!("archive-{index}"));
        guidance(&root, &output, &["edit-plan"]);
    }
    assert_eq!(snapshot(&archive), before);
}

// DECISION: D199
#[test]
fn ledger_edit_guidance_follows_the_file_the_file_system_finds() {
    let (_temp, root) = initialized(&["--skills", "guides"]);
    let state = fs::metadata(root.join("Ledger/State.md")).unwrap();
    let spellings = ["ledger/State.md", "LEDGER/state.MD"];
    for (index, written) in spellings.into_iter().enumerate() {
        let found = fs::metadata(root.join(written));
        let same_file = found.is_ok_and(|found| found.ino() == state.ino());
        let event = pre_tool(&root, "Write", json!({"file_path": written}));
        let output = in_session(&root, event, &format!("case-{index}"));
        let expected: &[&str] = if same_file { &["edit-state"] } else { &[] };
        guidance(&root, &output, expected);
    }
}

#[test]
fn nested_cwd_and_symlinks() {
    let (_temp, root) = project();
    let event = pre_tool(
        &root.join("Ledger"),
        "Edit",
        json!({"file_path": "Plan.md"}),
    );
    let guided = in_session(&root, event.clone(), "direct");
    assert!(guided.to_string().contains("edit-plan"), "{guided}");
    let path = root.join("Ledger/Plan.md");
    fs::remove_file(&path).unwrap();
    symlink(root.parent().unwrap().join("outside.md"), &path).unwrap();
    assert!(in_session(&root, event, "linked").is_null());
}

#[test]
fn storage_failure_keeps_recovery_context() {
    let (_temp, root) = project();
    let storage = root.join(".cache/runtime");
    fs::rename(&storage, root.join(".cache/moved")).unwrap();
    fs::write(&storage, "occupied").unwrap();
    let session = r#"{"hook_event_name": "SessionStart"}"#;
    let result = run(&mut runtime(&root, &["hook", "--agent", "codex"]), session);
    let reminders = storage.join("reminders");
    let recovery = [
        "Ledger/State.md",
        "FULL_REFRESH_REQUIRED",
        "Hook session state unavailable",
        reminders.to_str().unwrap(),
    ];
    for text in recovery {
        assert!(result.stdout.contains(text), "{text}: {}", result.stdout);
    }
    let stderr = &result.stderr;
    assert!(stderr.contains("session reminder state"), "{stderr}");
}

fn started(root: &Path, source: &str) -> String {
    let start = json!({"hook_event_name": "SessionStart", "source": source});
    let output = respond(root, &start);
    let context = output["hookSpecificOutput"]["additionalContext"].as_str();
    context.unwrap_or_else(|| panic!("{output}")).to_owned()
}

#[test]
fn every_renewed_context_receives_the_north_star_and_the_glossary_in_full() {
    let (_temp, root) = project();
    let north_star =
        "# North Star\n\nThe rig keeps code clear for twenty years.\n\nIt works alone.";
    let glossary = "# Glossary\n\n- **очередь** — очередь трекера, а не порядок работ";
    fs::write(root.join("Ledger/NorthStar.md"), format!("{north_star}\n")).unwrap();
    fs::write(root.join("Ledger/Glossary.md"), format!("{glossary}\n")).unwrap();
    for source in ["startup", "clear", "compact"] {
        let context = started(&root, source);
        for text in [north_star, glossary, "Resume from", "Ledger/State.md"] {
            assert!(context.contains(text), "{source}: {text}: {context}");
        }
    }
}

// DECISION: D242
#[test]
fn the_session_start_names_the_card_of_the_branch_or_else_the_plan() {
    let (_temp, root) = initialized(&["--key", "RIG"]);
    let card = "Ledger/Plan/RIG-7.md";
    write(&root, card, "# RIG-7 — Explain slow checks\n");
    let state = root.join("Ledger/State.md");
    let branches = [
        ("feature/slow-checks", root.join("Ledger/Plan")),
        ("feature/RIG-7-slow-checks", root.join(card)),
    ];
    for (branch, named) in branches {
        git(&root, &["switch", "-qc", branch]);
        let context = started(&root, "startup");
        let resumed = format!("Resume from {} and {}.", state.display(), named.display());
        assert!(context.contains(&resumed), "{branch}: {context}");
    }
}

#[test]
fn a_session_without_the_glossary_names_it_and_starts() {
    let (_temp, root) = project();
    let glossary = root.join("Ledger/Glossary.md");
    fs::remove_file(&glossary).unwrap();
    let context = started(&root, "startup");
    let missing = format!("{} is missing", glossary.display());
    for text in ["# North Star", missing.as_str(), "Resume from"] {
        assert!(context.contains(text), "{text}: {context}");
    }
}

#[test]
fn a_session_with_an_unreadable_north_star_names_it_and_starts() {
    let (_temp, root) = project();
    let north_star = root.join("Ledger/NorthStar.md");
    fs::remove_file(&north_star).unwrap();
    fs::create_dir(&north_star).unwrap();
    let context = started(&root, "startup");
    let unreadable = format!("{} is unreadable", north_star.display());
    for text in [unreadable.as_str(), "# Glossary", "Resume from"] {
        assert!(context.contains(text), "{text}: {context}");
    }
}
