use crate::shared::testing::{configure, git, invoke};
use serde_yaml_ng::{Mapping, Value};
use std::{
    fs,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
};
use tempfile::TempDir;

const TOOL: &str = "#!/bin/sh\necho report\n";
const CONTRACT: &str =
    "purpose: Produce project reports\nfiles:\n  report.sh: Summarize the project state\n";
const REPORT: &str = "report:
  purpose: Summarize the project state
  argv: [sh, tools/report.sh]
  doc: tools/README.md
";
const FINDINGS: [(&str, &str); 11] = [
    (
        "loose.sh",
        "tool is outside tools.locations: .agentrig/bin/**, .agentrig/hooks/*, tools/*",
    ),
    ("tools/stray", "tool is not registered"),
    ("tools/report.sh", "tool is not registered"),
    (
        ".agentrig/commands.yaml",
        "commands.report: purpose is required",
    ),
    (
        ".agentrig/commands.yaml",
        "commands.report: tools/moved.sh does not exist",
    ),
    (
        ".agentrig/tools.yaml",
        "tools.entries tools/gone/* matches no tool",
    ),
    (
        ".agentrig/tools.yaml",
        "tools.removed.old: tools/report.sh still exists",
    ),
    (
        ".agentrig/tools.yaml",
        "tools.removed.test: purpose and reason are required",
    ),
    (
        ".agentrig/tools.yaml",
        "tools.removed.test is also an active catalog command",
    ),
    (
        ".agentrig/tools.yaml",
        "tools.removed.test: revision does not contain tools/never.sh",
    ),
    ("tools/README.md", "references removed tool tools/report.sh"),
];

fn tools(root: &Path, edit: impl FnOnce(&mut Mapping)) {
    let path = root.join(".agentrig/tools.yaml");
    let mut settings: Value = serde_yaml_ng::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
    edit(settings.as_mapping_mut().unwrap());
    fs::write(path, serde_yaml_ng::to_string(&settings).unwrap()).unwrap();
}

fn append(settings: &mut Mapping, key: &str, item: &str) {
    let list = settings
        .get_mut(key)
        .and_then(Value::as_sequence_mut)
        .unwrap();
    list.push(Value::from(item));
}

fn consumer() -> (TempDir, PathBuf) {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().canonicalize().unwrap();
    let initialized = invoke(&root, &["init"]);
    assert_eq!(initialized.code, Some(0), "{}", initialized.output());
    fs::create_dir(root.join("tools")).unwrap();
    fs::write(root.join("tools/report.sh"), TOOL).unwrap();
    fs::write(root.join("tools/architecture.yaml"), CONTRACT).unwrap();
    fs::write(root.join("tools/README.md"), "# Reports\n").unwrap();
    configure(&root, "commands", REPORT);
    tools(&root, |settings| append(settings, "locations", "tools/*"));
    (temp, root)
}

fn commit(root: &Path) -> String {
    git(root, &["add", "-A"]);
    let identity = [
        "-c",
        "user.name=Test",
        "-c",
        "user.email=test@example.invalid",
    ];
    let unhooked = ["-c", "core.hooksPath=/dev/null"];
    git(
        root,
        &[&unhooked[..], &identity, &["commit", "-qm", "state"]].concat(),
    );
    git(root, &["rev-parse", "HEAD"])
}

fn removal(purpose: &str, reason: &str, path: &str, revision: Option<&str>) -> Value {
    let mut record = Mapping::new();
    for (key, value) in [("purpose", purpose), ("reason", reason), ("path", path)] {
        record.insert(key.into(), value.into());
    }
    if let Some(revision) = revision {
        record.insert("revision".into(), revision.into());
    }
    Value::Mapping(record)
}

fn following<'a>(lines: &[&'a str], heading: &str, count: usize) -> Vec<&'a str> {
    let start = lines.iter().position(|line| *line == heading).unwrap() + 1;
    lines[start..start + count].to_vec()
}

fn retired(root: &Path) -> String {
    fs::write(root.join("tools/old.sh"), TOOL).unwrap();
    let revision = commit(root);
    fs::remove_file(root.join("tools/old.sh")).unwrap();
    let old = removal(
        "Print the old report",
        "replaced by report",
        "tools/old.sh",
        Some(&revision),
    );
    let probe = removal(
        "Try a report layout",
        "experiment ended",
        "tools/probe.sh",
        None,
    );
    let removed = Mapping::from_iter([(Value::from("old"), old), (Value::from("probe"), probe)]);
    tools(root, |settings| {
        settings.insert("removed".into(), Value::Mapping(removed));
    });
    revision
}

fn capabilities(lines: &[&str]) {
    let report = [
        "  just run report",
        "  tools/report.sh: Summarize the project state; owner tools: Produce project reports",
        "  doc: tools/README.md",
    ];
    assert_eq!(
        following(lines, "report: Summarize the project state", 3),
        report
    );
    let read = [
        "  just run read -- PROGRAM ARGS; programs: cat, diff, find, git, grep, head, ls, rg, sed, stat, tail, wc",
        "report: Summarize the project state",
    ];
    let heading = "read: Inspect files and Git history with configured programs";
    assert_eq!(following(lines, heading, 2), read);
    let test = following(lines, "test: Run the project tests", 2);
    assert_eq!(test[1], "  runs python3 -m pytest Project");
    assert!(
        lines
            .iter()
            .any(|line| line.starts_with(".agentrig/hooks/pre-commit: "))
    );
}

#[test]
fn registry_lists_capabilities_then_removed_tools() {
    let (_temp, root) = consumer();
    let revision = retired(&root);
    let listing = invoke(&root, &["commands"]);
    assert_eq!(listing.code, Some(0), "{}", listing.output());
    let lines: Vec<&str> = listing.stdout.lines().collect();
    capabilities(&lines);
    let restore =
        format!("  tools/old.sh: replaced by report; restore: git show {revision}:tools/old.sh");
    let history = [
        "old: Print the old report",
        &restore,
        "probe: Try a report layout",
        "  tools/probe.sh: experiment ended; never committed; not recoverable",
    ];
    assert_eq!(following(&lines, "Removed:", 4), history);
    assert_eq!(lines.last(), Some(&history[3]));
    let shown = git(&root, &["show", &format!("{revision}:tools/old.sh")]);
    assert_eq!(shown, TOOL.trim_end());
}

#[test]
fn tools_check_reports_structure_before_merge() {
    let (_temp, root) = consumer();
    let initial = commit(&root);
    assert_eq!(invoke(&root, &["check", "--only", "tools"]).code, Some(0));
    fs::write(root.join("loose.sh"), TOOL).unwrap();
    let stray = root.join("tools/stray");
    fs::write(&stray, "data").unwrap();
    fs::set_permissions(&stray, fs::Permissions::from_mode(0o755)).unwrap();
    fs::write(root.join("tools/README.md"), "Run tools/report.sh\n").unwrap();
    configure(&root, "commands", "report:\n  argv: [sh, tools/moved.sh]\n");
    tools(&root, |settings| {
        append(settings, "entries", "tools/gone/*");
        let old = removal("Report", "moved", "tools/report.sh", None);
        let never = removal("", "", "tools/never.sh", Some(&initial));
        let removed = Mapping::from_iter([(Value::from("old"), old), (Value::from("test"), never)]);
        settings.insert("removed".into(), Value::Mapping(removed));
    });
    let result = invoke(&root, &["check", "--only", "tools"]);
    assert_eq!(result.code, Some(1), "{}", result.output());
    for (path, message) in FINDINGS {
        let line = format!("  {path}: {message}");
        assert!(result.stdout.contains(&line), "{line}\n{}", result.stdout);
    }
    assert!(
        result.stderr.contains("develop-tools/SKILL.md"),
        "{}",
        result.stderr
    );
    git(&root, &["add", "-A"]);
    let staged = invoke(&root, &["check", "--staged", "--only", "tools"]);
    let skipped = staged.stdout.contains("SKIP [tools]: checked before merge");
    assert!(skipped, "{}", staged.output());
}

#[test]
fn tools_check_reads_the_checked_revision() {
    let (_temp, root) = consumer();
    let loose = root.join("loose.sh");
    fs::write(&loose, TOOL).unwrap();
    let stray = commit(&root);
    fs::remove_file(&loose).unwrap();
    commit(&root);
    fs::write(&loose, TOOL).unwrap();
    let passing = invoke(&root, &["check", "--revision", "HEAD", "--only", "tools"]);
    assert_eq!(passing.code, Some(0), "{}", passing.output());
    let failing = invoke(&root, &["check", "--revision", &stray, "--only", "tools"]);
    assert_eq!(failing.code, Some(1), "{}", failing.output());
    assert!(
        failing
            .stdout
            .contains("loose.sh: tool is outside tools.locations")
    );
}
