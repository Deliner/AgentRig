// DECISION: D021
use anyhow::Result;
use serde_json::json;

pub fn template(skills: &str, sources: &[String]) -> Result<String> {
    let mut rules = Vec::new();
    for &(id, kind, target, skill, warning, error) in DEFAULT_RULES {
        let skill = format!("{skills}/{skill}/SKILL.md");
        let mut rule = json!({
            "id": id, "kind": kind, "target": target,
            "include": sources,
            "warning_skill": skill, "error_skill": skill,
        });
        let named_condition = kind == "named-if-condition";
        if named_condition {
            rule["level"] = json!("error");
        } else {
            if let Some(warning) = warning {
                rule["warning"] = json!(warning);
            }
            if let Some(error) = error {
                rule["error"] = json!(error);
            }
        }
        let syntax_rule = crate::lint::rules::syntax(kind);
        if syntax_rule {
            rule["extensions"] = json!([".rs", ".py", ".pyi"]);
        }
        rules.push(rule);
    }
    let config = json!({
        "version": 1,
        "config_skill": format!("{skills}/repair/SKILL.md"),
        "exclude": [".git/**", ".worker/bin/**", ".worker/runtime/**", "**/target/**", "**/__pycache__/**", "**/.pytest_cache/**"],
        "rules": rules,
    });
    Ok(toml::to_string_pretty(&config)?)
}

type DefaultRule = (
    &'static str,
    &'static str,
    &'static str,
    &'static str,
    Option<u64>,
    Option<u64>,
);
const DEFAULT_RULES: &[DefaultRule] = &[
    (
        "file-size",
        "nonblank-lines",
        "file",
        "refactor-large-file",
        Some(300),
        Some(500),
    ),
    (
        "directory-size",
        "directory-entries",
        "directory",
        "refactor-large-directory",
        Some(10),
        Some(15),
    ),
    (
        "function-size",
        "function-lines",
        "file",
        "refactor-long-function",
        None,
        Some(40),
    ),
    (
        "parameters",
        "parameter-count",
        "file",
        "reduce-parameters",
        None,
        Some(4),
    ),
    (
        "named-if",
        "named-if-condition",
        "file",
        "name-if-condition",
        None,
        None,
    ),
];
