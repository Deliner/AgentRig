use super::{command, configured, jobs};
use crate::shared::testing::{Run, git, invoke, run, runtime};
use serde_json::json;
use std::{
    fs,
    path::{Path, PathBuf},
};

const NOISE: &str = "i=0; while [ $i -lt \"$2\" ]; do echo noise $i; i=$((i+1)); done; \
echo FAILED case; echo done $1; exit $1";
const VIEWS: &str = "  accepts_args: true
  output:
    success: {tail: 1}
    failure: {matching: [FAILED], tail: 1}
";
const REFUSALS: [(&[&str], &str); 5] = [
    (
        &["read", "--", "sh", "-c", "ls"],
        "read does not run sh; configured programs: cat, diff, find, git, grep, head, ls, rg, sed, stat, tail, \
wc. Project commands: just list. Temporary code: just scratch -- TEXT",
    ),
    (
        &["write", "--", "python3", "-c", "pass"],
        "write does not run python3; configured",
    ),
    (
        &["read", "--", "git", "-c", "core.pager=sh", "log"],
        "read runs git only with: status,",
    ),
    (
        &["read", "--", "find", ".", "-exec", "cat", "{}", "+"],
        "read does not run find with -exec",
    ),
    (
        &["write", "--", "git", "rebase", "--exec", "true", "main"],
        "does not run git with --exec",
    ),
];

fn initialized() -> (tempfile::TempDir, PathBuf) {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().canonicalize().unwrap();
    let result = invoke(&root, &["init"]);
    assert_eq!(result.code, Some(0), "{}", result.output());
    (temp, root)
}

fn summary(root: &Path, code: &str, noise: &str) -> (Option<i32>, Vec<String>, PathBuf) {
    let result = invoke(root, &["run", "noisy", "--", code, noise]);
    let mut lines: Vec<String> = result.stdout.lines().map(str::to_owned).collect();
    let status = lines.pop().unwrap();
    let prefix = format!("noisy: exit {}; log: ", result.code.unwrap());
    let log = status
        .strip_prefix(&prefix)
        .unwrap_or_else(|| panic!("{}", result.stdout));
    (result.code, lines, PathBuf::from(log))
}

fn input(root: &Path, args: &[&str], text: &str) -> Run {
    run(&mut runtime(root, args), text)
}

#[test]
fn configuration_and_command_streams() {
    let (_temp, root) = configured("");
    assert_eq!(invoke(&root, &["config-check"]).code, Some(0));
    let echoed = input(
        &root,
        &["run", "echo", "--", "a b", "$(no-shell)"],
        "input data",
    );
    assert_eq!(echoed.code, Some(0), "{}", echoed.output());
    assert!(
        echoed.stdout.contains("a b\n$(no-shell)\n"),
        "{}",
        echoed.stdout
    );
    assert!(echoed.stdout.contains("input data"), "{}", echoed.stdout);
    assert_eq!(echoed.stderr, "stderr\n");
    assert_eq!(invoke(&root, &["run", "fail"]).code, Some(23));
    let mut codes: Vec<_> = jobs(&root)
        .iter()
        .map(|row| row["exit_code"].clone())
        .collect();
    codes.sort_by_key(|code| code.as_i64());
    assert_eq!(codes, [json!(0), json!(23)]);
    assert!(!root.join(".runtime/commands.jsonl").exists());
    assert!(invoke(&root, &["report"]).stdout.contains("fail 1 1"));
    assert_eq!(invoke(&root, &["run", "unknown"]).code, Some(2));
    assert_eq!(invoke(&root, &["run", "fail", "extra"]).code, Some(2));
}

#[test]
fn read_commands_cannot_write() {
    let (_temp, root) = configured("");
    let target = root.join("src/protected");
    fs::write(&target, "original").unwrap();
    let writing = [
        "run",
        "read",
        "--",
        "sh",
        "-c",
        "echo replaced > src/protected",
    ];
    assert_ne!(invoke(&root, &writing).code, Some(0));
    assert_eq!(fs::read_to_string(&target).unwrap(), "original");
    let reading = invoke(&root, &["run", "read", "--", "cat", "src/protected"]);
    assert_eq!(reading.code, Some(0), "{}", reading.output());
    assert_eq!(reading.stdout, "original");
}

#[test]
fn base_programs_and_argument_forms() {
    let (_temp, root) = initialized();
    assert_eq!(invoke(&root, &["run", "read", "--", "ls"]).code, Some(0));
    assert_eq!(
        invoke(&root, &["run", "write", "--", "touch", "made"]).code,
        Some(0)
    );
    assert!(root.join("made").is_file());
    for (args, message) in REFUSALS {
        let result = invoke(&root, &[&["run"], args].concat());
        assert_eq!(result.code, Some(2), "{args:?}");
        assert!(result.stderr.contains(message), "{}", result.stderr);
    }
    let commands = root.join(".agentrig/commands.yaml");
    fs::write(commands, "read:\n  argv: []\n  accepts_args: true\n").unwrap();
    let invalid = invoke(&root, &["config-check"]);
    assert_eq!(invalid.code, Some(2));
    let named = "commands.read: empty argv requires programs";
    assert!(invalid.stderr.contains(named), "{}", invalid.stderr);
}

#[test]
fn configured_views_compact_command_output() {
    let noisy = command("noisy", &["sh", "-c", NOISE, "noisy"]) + VIEWS;
    let (_temp, root) = configured(&noisy);
    let (code, lines, log) = summary(&root, "0", "5");
    assert_eq!((code, lines), (Some(0), vec!["done 0".to_owned()]));
    assert_eq!(summary(&root, "0", "500").1, ["done 0"]);
    let captured = fs::read_to_string(log.join("stdout.log")).unwrap();
    assert!(
        captured.contains("noise 4\nFAILED case\ndone 0\n"),
        "{captured}"
    );
    let (code, lines, _) = summary(&root, "3", "500");
    assert_eq!(
        (code, lines),
        (Some(3), vec!["FAILED case".to_owned(), "done 3".to_owned()])
    );
}

#[test]
fn scratch_runs_temporary_code_in_its_task_area() {
    let (_temp, root) = initialized();
    git(&root, &["symbolic-ref", "HEAD", "refs/heads/feature/probe"]);
    let text = "echo found > \"$AGENTRIG_SCRATCH/results/out\" && pwd";
    let result = invoke(&root, &["scratch", "--", text]);
    assert_eq!(result.code, Some(0), "{}", result.output());
    assert_eq!(result.stdout.trim(), root.to_str().unwrap());
    let area = root.join(".agentrig/runtime/scratch/feature/probe");
    assert_eq!(
        fs::read_to_string(area.join("results/out")).unwrap(),
        "found\n"
    );
    assert_eq!(invoke(&root, &["scratch", "--", "exit 7"]).code, Some(7));
    let rows = jobs(&root);
    let record = rows
        .iter()
        .find(|row| row["argv"] == json!(["sh", "-c", text]))
        .unwrap();
    let identity = (&record["command"], &record["branch"], &record["lifetime"]);
    assert_eq!(
        identity,
        (&json!("scratch"), &json!("feature/probe"), &json!("task"))
    );
}
