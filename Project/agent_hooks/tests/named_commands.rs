use super::{hook, project, reason};
use crate::shared::testing::repository;
use serde_json::{Value, json};
use std::{
    fs,
    path::{Path, PathBuf},
};

const TEXTS: [&str; 3] = ["AGENTS.md", "README.md", "SKILL.md"];
const NAMED_RECIPES: [&str; 1] = ["just write"];
const EXAMPLES: [(&str, &str); 14] = [
    ("...", "x"),
    ("ARGS", "x"),
    ("FILTER", "x"),
    ("<name>", "x"),
    ("NAME", "test"),
    ("COMMAND", "test"),
    ("CHECK_ID", "lint"),
    ("ID", "run-x"),
    ("KIND", "function-lines"),
    ("PATH", "Project"),
    ("DIR", "Project"),
    ("OLD", "HEAD"),
    ("<base>", "main"),
    ("<plan-commit>", "HEAD"),
];

fn texts(dir: &Path, found: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(dir).unwrap() {
        let entry = entry.unwrap();
        let name = entry.file_name().to_string_lossy().into_owned();
        let hidden = name.starts_with('.');
        let directory = entry.file_type().unwrap().is_dir();
        let text = TEXTS.contains(&name.as_str());
        if hidden {
            continue;
        }
        if directory {
            texts(&entry.path(), found);
        } else if text {
            found.push(entry.path());
        }
    }
}

fn commands(text: &str) -> Vec<String> {
    let mut found = Vec::new();
    let mut fenced = false;
    let mut prose = String::new();
    for line in text.lines() {
        let fence = line.trim_start().starts_with("```");
        let command = line.trim();
        let fenced_command = fenced && command.starts_with("just ");
        let prose_line = !fenced;
        if fence {
            fenced.toggle();
        } else if fenced_command {
            found.push(command.to_owned());
        } else if prose_line {
            prose.push_str(line);
            prose.push(' ');
        }
    }
    let spans = prose.split('`').skip(1).step_by(2);
    let words = spans.map(|span| span.split_whitespace().collect::<Vec<_>>().join(" "));
    found.extend(words.filter(|span| span.starts_with("just ")));
    found
}

fn example(command: &str) -> String {
    let writes = command.starts_with("just write ");
    let program = if writes { "touch" } else { "cat" };
    let substitute = |word| match word {
        "PROGRAM" => program,
        _ => EXAMPLES
            .iter()
            .find(|(placeholder, _)| *placeholder == word)
            .map_or(word, |(_, value)| *value),
    };
    let words: Vec<&str> = command.split_whitespace().map(substitute).collect();
    words.join(" ")
}

fn verdict(root: &Path, command: &str) -> Value {
    let input = json!({"command": command});
    let _delivered_route_stops = hook(root, "Bash", input.clone());
    hook(root, "Bash", input)
}

fn refused(root: &Path, paths: &[PathBuf]) -> Vec<String> {
    let mut refused = Vec::new();
    for path in paths {
        let text = fs::read_to_string(path).unwrap();
        let named = commands(&text);
        let runnable = named
            .iter()
            .filter(|command| !NAMED_RECIPES.contains(&command.as_str()));
        for command in runnable {
            let output = verdict(root, &example(command));
            let denied = output["hookSpecificOutput"]["permissionDecision"] == "deny";
            if denied {
                let why = reason(&output);
                refused.push(format!("{}: {command}: {why}", path.display()));
            }
        }
    }
    refused
}

#[test]
fn every_command_the_rig_texts_name_passes_the_hook_with_the_rig_catalog() {
    let (_temp, root) = project();
    let catalog = ".agentrig/commands.yaml";
    fs::copy(repository().join(catalog), root.join(catalog)).unwrap();
    let agents = fs::read_to_string(repository().join("AGENTS.md")).unwrap();
    assert!(commands(&agents).contains(&"just feature-merge".to_owned()));
    let mut paths = Vec::new();
    texts(repository(), &mut paths);
    let refused = refused(&root, &paths);
    assert!(refused.is_empty(), "{refused:#?}");
}

#[test]
fn every_command_the_installed_texts_name_passes_the_hook() {
    let (_temp, root) = project();
    let agents = root.join("AGENTS.md");
    let instructions = fs::read_to_string(&agents).unwrap();
    assert!(commands(&instructions).contains(&"just feature-merge".to_owned()));
    let mut paths = vec![agents];
    texts(&root.join(".agentrig/skills"), &mut paths);
    let refused = refused(&root, &paths);
    assert!(refused.is_empty(), "{refused:#?}");
}
