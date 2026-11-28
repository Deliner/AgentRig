// DECISION: D189
use crate::project_config;
use crate::shared::testing::{git, invoke, repository, write};
use serde::Deserialize;
use serde_json::{Map, Value};
use std::{
    fs,
    path::{Path, PathBuf},
};
use tempfile::TempDir;

const COMPARED: [&str; 4] = [".agentrig", "justfile", ".codex", ".claude"];
const LISTED: &str = include_str!("rig_differences.yaml");
const PERSONAL_SETTINGS: &str = "settings.local.json";

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Listed {
    reason: String,
    at: Vec<Vec<String>>,
}

fn consumer() -> (TempDir, PathBuf) {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().canonicalize().unwrap();
    git(&root, &["init", "-q", "-b", "master"]);
    git(&root, &["remote", "add", "origin", "../origin.git"]);
    let manifest = "Project/Cargo.toml";
    fs::create_dir_all(root.join("Project")).unwrap();
    fs::copy(repository().join(manifest), root.join(manifest)).unwrap();
    let lint = parsed(&repository().join(".agentrig/lint.yaml"));
    let rules = lint["rules"].as_array().unwrap().iter();
    let architecture = rules.into_iter().find(|rule| rule["id"] == "architecture");
    for crate_root in architecture.unwrap()["architecture"]["rust_roots"]
        .as_array()
        .unwrap()
    {
        write(&root, crate_root.as_str().unwrap(), "");
    }
    let args = [
        "init",
        "--key",
        "RIG",
        "--language",
        "rust",
        "--base",
        "master",
    ];
    let initialized = invoke(&root, &args);
    assert_eq!(initialized.code, Some(0), "{}", initialized.output());
    (temp, root)
}

fn parsed(path: &Path) -> Value {
    let name = path.to_string_lossy();
    let Ok(text) = fs::read_to_string(path) else {
        return Value::from("binary");
    };
    let yaml = name.ends_with(".yaml");
    if yaml {
        return serde_yaml_ng::from_str(&text).unwrap();
    }
    let json = name.ends_with(".json");
    if json {
        return serde_json::from_str(&text).unwrap();
    }
    let toml = name.ends_with(".toml");
    if toml {
        return toml::from_str(&text).unwrap();
    }
    let lines = text.lines().map(str::trim).filter(|line| {
        let comment = line.starts_with('#') && !line.starts_with("#!");
        !line.is_empty() && !comment
    });
    Value::from(lines.collect::<Vec<_>>())
}

fn node(path: &Path) -> Value {
    let metadata = fs::symlink_metadata(path).unwrap();
    let link = metadata.is_symlink();
    if link {
        let target = fs::read_link(path).unwrap();
        return Value::from(format!("link {}", target.display()));
    }
    let file = metadata.is_file();
    if file {
        return parsed(path);
    }
    let entries = fs::read_dir(path).unwrap().map(|entry| entry.unwrap());
    let shared = entries.filter(|entry| entry.file_name() != PERSONAL_SETTINGS);
    let named = shared.map(|entry| {
        (
            entry.file_name().to_string_lossy().into(),
            node(&entry.path()),
        )
    });
    Value::Object(named.collect())
}

fn setup(root: &Path) -> Value {
    let present = COMPARED.into_iter().filter(|name| root.join(name).exists());
    let mut tree: Map<String, Value> = present
        .map(|name| (name.into(), node(&root.join(name))))
        .collect();
    let config = project_config::read(root).unwrap();
    for module in project_config::files(&config).unwrap() {
        let (service, file) = module.path.split_once('/').unwrap();
        tree[service][file] = serde_yaml_ng::from_str(&module.source).unwrap();
    }
    Value::Object(tree)
}

fn scalar(value: &Value) -> String {
    value
        .as_str()
        .map_or_else(|| value.to_string(), str::to_owned)
}

fn keyed(items: &[Value]) -> Option<Map<String, Value>> {
    let ids = items
        .iter()
        .map(|item| Some((scalar(item.get("id")?), item.clone())));
    ids.collect()
}

fn compare_lists(at: &[String], rig: &[Value], init: &[Value], found: &mut Vec<Vec<String>>) {
    let records = rig.iter().chain(init).all(Value::is_object);
    if records {
        let index = |items: &[Value]| -> Map<String, Value> {
            keyed(items).unwrap_or_else(|| {
                items
                    .iter()
                    .enumerate()
                    .map(|(i, item)| (i.to_string(), item.clone()))
                    .collect()
            })
        };
        return compare(
            at,
            &Value::Object(index(rig)),
            &Value::Object(index(init)),
            found,
        );
    }
    let names = |items: &[Value]| items.iter().map(scalar).collect::<Vec<_>>();
    let (rig, init) = (names(rig), names(init));
    let only_rig = rig.iter().filter(|item| !init.contains(item));
    let only_init = init.iter().filter(|item| !rig.contains(item));
    let extra: Vec<_> = only_rig
        .chain(only_init)
        .map(|item| [at, &[item.clone()]].concat())
        .collect();
    let reordered = extra.is_empty() && rig != init;
    if reordered {
        found.push(at.to_vec());
    }
    found.extend(extra);
}

fn compare(at: &[String], rig: &Value, init: &Value, found: &mut Vec<Vec<String>>) {
    match (rig, init) {
        (Value::Object(rig), Value::Object(init)) => {
            for key in rig
                .keys()
                .chain(init.keys().filter(|key| !rig.contains_key(*key)))
            {
                let path = [at, &[key.clone()]].concat();
                match (rig.get(key), init.get(key)) {
                    (Some(rig), Some(init)) => compare(&path, rig, init, found),
                    _ => found.push(path),
                }
            }
        }
        (Value::Array(rig), Value::Array(init)) => compare_lists(at, rig, init, found),
        _ => {
            let differ = rig != init;
            if differ {
                found.push(at.to_vec());
            }
        }
    }
}

fn uncovered<'a>(paths: &'a [Vec<String>], prefixes: &[&Vec<String>]) -> Vec<String> {
    let covered = |path: &&'a Vec<String>| prefixes.iter().any(|prefix| path.starts_with(prefix));
    let open = paths.iter().filter(|path| !covered(path));
    open.map(|path| path.join(" / ")).collect()
}

// INVARIANT: I023
#[test]
fn init_matches_the_rig_setup_except_listed_differences() {
    let (_temp, root) = consumer();
    let mut found = Vec::new();
    compare(&[], &setup(repository()), &setup(&root), &mut found);
    let listed: Vec<Listed> = serde_yaml_ng::from_str(LISTED).unwrap();
    let reasoned = listed.iter().all(|entry| !entry.reason.trim().is_empty());
    assert!(reasoned, "every listed difference needs its reason");
    let prefixes: Vec<_> = listed.iter().flat_map(|entry| &entry.at).collect();
    let unexplained = uncovered(&found, &prefixes);
    assert!(
        unexplained.is_empty(),
        "differences missing from rig_differences.yaml: {unexplained:#?}"
    );
    let unused = prefixes
        .iter()
        .filter(|prefix| !found.iter().any(|path| path.starts_with(prefix)));
    let unused: Vec<_> = unused.map(|prefix| prefix.join(" / ")).collect();
    assert!(
        unused.is_empty(),
        "listed differences the rig no longer has: {unused:#?}"
    );
}
