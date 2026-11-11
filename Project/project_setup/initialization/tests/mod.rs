mod claude;
mod layout;
mod wizard;

use super::init;
use crate::shared::testing::repository;
use serde_yaml_ng::Value;
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::Path,
};

// DECISION: D053
// DECISION: D158
const WORKER_LOCAL_CHECKS: [&str; 8] = [
    "build",
    "rustfmt",
    "clippy",
    "ruff-format",
    "ruff",
    "mypy",
    "typos",
    "vulture",
];

type Capabilities = BTreeMap<&'static str, BTreeSet<String>>;

fn yaml(path: &Path) -> Value {
    serde_yaml_ng::from_str(&fs::read_to_string(path).unwrap()).unwrap()
}

fn text(value: &Value) -> String {
    value.as_str().unwrap().to_owned()
}

fn names(entries: &Value, name: impl Fn(&Value) -> String) -> BTreeSet<String> {
    entries.as_sequence().unwrap().iter().map(name).collect()
}

fn skills(directory: &Path, found: &mut BTreeSet<String>) {
    for entry in fs::read_dir(directory).unwrap() {
        let path = entry.unwrap().path();
        let skill = path.ends_with("SKILL.md");
        let nested = path.is_dir();
        if skill {
            found.insert(directory.file_name().unwrap().to_string_lossy().into());
        } else if nested {
            skills(&path, found);
        }
    }
}

fn check_capability(check: &Value) -> String {
    let command = check["kind"] == "command";
    let field = if command { "command" } else { "kind" };
    text(&check[field])
}

fn capabilities(service: &Path, skill_root: &Path, worker_local: &[&str]) -> Capabilities {
    let rules = &yaml(&service.join("lint.yaml"))["rules"];
    let routes = &yaml(&service.join("hooks.yaml"))["routes"];
    let checks = yaml(&service.join("checks.yaml"));
    let route_skill = |route: &Value| {
        let skill = text(&route["skill"]);
        skill.rsplit('/').nth(1).unwrap().to_owned()
    };
    let mut checks = names(&checks, check_capability);
    checks.retain(|check| !worker_local.contains(&check.as_str()));
    let mut skill_names = BTreeSet::new();
    skills(skill_root, &mut skill_names);
    BTreeMap::from([
        ("rule-kinds", names(rules, |rule| text(&rule["kind"]))),
        ("skills", skill_names),
        ("routes", names(routes, route_skill)),
        ("checks", checks),
    ])
}

fn divergences(delivered: &Capabilities, live: &Capabilities) -> Vec<String> {
    let mut report = Vec::new();
    for (dimension, shipped) in delivered {
        let present = &live[dimension];
        let only_delivered: Vec<_> = shipped.difference(present).collect();
        let only_live: Vec<_> = present.difference(shipped).collect();
        let delivered_extra = !only_delivered.is_empty();
        if delivered_extra {
            report.push(format!(
                "{dimension} only in init delivery: {only_delivered:?}"
            ));
        }
        let live_extra = !only_live.is_empty();
        if live_extra {
            report.push(format!("{dimension} only in repository: {only_live:?}"));
        }
    }
    report
}

#[test]
fn init_collision_preserves_existing_files() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().canonicalize().unwrap();
    let path = root.join("justfile");
    fs::write(&path, "user contents").unwrap();
    let error = init(&root, &[]).unwrap_err().to_string();
    assert!(error.contains("collision"), "{error}");
    let entries: Vec<_> = fs::read_dir(&root)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .collect();
    assert_eq!(entries, [path.clone()]);
    assert_eq!(fs::read_to_string(path).unwrap(), "user contents");
}

// INVARIANT: I020
#[test]
fn init_delivers_the_universal_capabilities_of_this_repository() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().canonicalize().unwrap();
    let language = ["--language".to_owned(), "python".to_owned()];
    assert_eq!(init(&root, &language).unwrap(), 0);
    let service = root.join(".agentrig");
    let delivered = capabilities(&service, &service.join("skills"), &WORKER_LOCAL_CHECKS);
    let live_service = repository().join(".agentrig");
    let live_skills = repository().join("Project/shared/skills");
    let live = capabilities(&live_service, &live_skills, &WORKER_LOCAL_CHECKS);
    let report = divergences(&delivered, &live);
    assert!(report.is_empty(), "{report:?}");
    let exposed = capabilities(&live_service, &live_skills, &[]);
    let toolchain = divergences(&delivered, &exposed);
    let reported = toolchain.iter().any(|message| message.contains("build"));
    assert!(reported, "{toolchain:?}");
    for dimension in ["rule-kinds", "skills", "routes"] {
        let mut one_sided = live.clone();
        one_sided.get_mut(dimension).unwrap().insert("demo".into());
        let report = divergences(&delivered, &one_sided);
        let named = |message: &String| message.contains(dimension) && message.contains("demo");
        assert!(report.iter().any(named), "{report:?}");
    }
}
