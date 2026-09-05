use anyhow::Result;
use serde_json::json;

pub fn template(skills: &str, source: &str) -> Result<String> {
    let mut rules = Vec::new();
    for (id, kind, target, skill, warning, error) in [
        (
            "file-size",
            "nonblank-lines",
            "file",
            "split-large-file",
            300,
            Some(500),
        ),
        (
            "directory-size",
            "directory-entries",
            "directory",
            "organize-directory",
            10,
            Some(15),
        ),
        (
            "function-size",
            "function-lines",
            "file",
            "refactor-long-function",
            40,
            None,
        ),
        (
            "parameters",
            "parameter-count",
            "file",
            "reduce-parameters",
            4,
            None,
        ),
        (
            "named-if",
            "named-if-condition",
            "file",
            "name-if-condition",
            0,
            None,
        ),
    ] {
        let skill = format!("{skills}/{skill}/SKILL.md");
        let mut rule = json!({
            "id": id, "kind": kind, "target": target,
            "include": [format!("{source}/**")],
            "warning_skill": skill, "error_skill": skill,
        });
        if kind == "named-if-condition" {
            rule["level"] = json!("warning");
        } else {
            rule["warning"] = json!(warning);
            if let Some(error) = error {
                rule["error"] = json!(error);
            }
        }
        if ["named-if-condition", "function-lines", "parameter-count"].contains(&kind) {
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
