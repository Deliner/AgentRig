use super::architecture::binaries;
use super::consumer::{CONFIG, command_with, consumer, lint, lint_with, write};
use crate::shared::testing::{launch, run, worker};
use serde_json::{Value, json};
use std::{fs, path::Path};

const OVERRIDES: &str = r#"
    overrides:
      - include: ["src/**"]
        warning: 7
        error: 9
      - include: ["src/special.py"]
        error: 12
"#;
const INVALID_OVERRIDE: &str = r#"
    overrides:
      - include: ["src/**"]
        warning: 9
"#;
const REASONS: [(&str, &str); 4] = [
    ("enabled: false\n", "disabled"),
    ("exclude: [\"src/example.py\"]\n", "rule exclude matches"),
    ("extensions: [\".rs\"]\n", "extension does not match"),
    ("include: [\"other/**\"]\n", "include does not match"),
];

fn explain(binary: &Path, root: &Path, path: &str) -> Value {
    let result = command_with(binary, root, "lint-explain", &[path, "--json"]);
    assert_eq!(result.code, Some(0), "{}", result.output());
    serde_json::from_str(&result.stdout).unwrap()
}

fn reason(root: &Path, path: &str) -> String {
    let report = explain(worker(), root, path);
    report["rules"][0]["reason"].as_str().unwrap().to_owned()
}

#[test]
fn explain_effective_overrides() {
    for binary in binaries() {
        let (_temp, root) = consumer(&format!("{CONFIG}{OVERRIDES}"));
        let text = "line\n".repeat(13);
        write(&root, "src/special.py", &text);
        let report = explain(&binary, &root, "./src/special.py");
        let row = &report["rules"][0];
        assert_eq!(row["selected"], true, "{row}");
        assert_eq!(row["reason"], "selected");
        let limits = [&row["warning"], &row["error"], &row["matched_overrides"]];
        assert_eq!(limits, [&json!(7), &json!(12), &json!([0, 1])]);
        let (code, findings) = lint_with(&binary, &root);
        assert_eq!(code, Some(1));
        assert_eq!(findings[0]["limit"], row["error"]);
        assert_eq!(
            fs::read_to_string(root.join("src/special.py")).unwrap(),
            text
        );
    }
}

#[test]
fn explain_selection_reasons() {
    for (setting, expected) in REASONS {
        let key = format!("{}:", setting.split(':').next().unwrap());
        let retained: Vec<&str> = CONFIG
            .lines()
            .filter(|line| !line.trim().starts_with(&key))
            .collect();
        let (_temp, root) = consumer(&format!("{}\n    {setting}", retained.join("\n")));
        write(&root, "src/example.py", &"line\n".repeat(8));
        let row = &explain(worker(), &root, "src/example.py")["rules"][0];
        assert_eq!(row["selected"], false, "{row}");
        assert_eq!(row["reason"], expected);
        assert_eq!(lint(&root), (Some(0), vec![]));
    }
}

#[test]
fn explain_inventory_and_global_exclusions() {
    let (_temp, root) = consumer(&format!("exclude: [\"src/hidden.py\"]\n{CONFIG}"));
    write(&root, "src/hidden.py", &"line\n".repeat(8));
    assert_eq!(reason(&root, "src/hidden.py"), "global exclude matches");
    assert!(reason(&root, "src/missing.py").contains("absent"));
    write(&root, "src/visible.py", "");
    assert_eq!(reason(&root, "src"), "target type does not match");
}

#[test]
fn explain_rejects_invalid_effective_policy() {
    let (_temp, root) = consumer(&format!("{CONFIG}{INVALID_OVERRIDE}"));
    write(&root, "src/example.py", "");
    let args = ["lint-explain", "src/example.py"];
    let result = run(&mut launch(worker(), &root, &args), "");
    assert_eq!(result.code, Some(2), "{}", result.output());
    let named = result.stderr.contains("invalid effective thresholds");
    assert!(named, "{}", result.stderr);
}
