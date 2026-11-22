// DECISION: D005
use super::{hook, project, reason};
use crate::shared::testing::{configure, repository};
use serde_json::{Value, json};
use std::{fs, path::Path};

const BYPASSES: [&str; 7] = [
    "git status",
    "just unknown",
    "just list && git status",
    r#"just run write echo "$(pwd)""#,
    "just --justfile other list",
    "just list\npwd",
    "just list write echo bypass",
];
const WRAPPERS: [&str; 2] = [
    "just run write -- sh -c 'git commit -m x'",
    "just read -- python3 tool.py",
];
const LISTINGS: [&str; 9] = [
    "just --list",
    "just list",
    "just lint-rule function-lines --example",
    "just lint-explain src/example.py --json",
    "just lint-edges src --json",
    "just lint-tree src",
    "just lint-structure src",
    "just run read -- git status",
    "just run write -- touch ok",
];

// DECISION: D201
const CODE_BEFORE_JUST: [&str; 6] = [
    "just read -- cat =(touch x)",
    "just read -- ls *(e:'touch x':)",
    "just read -- cat >(touch x)",
    r#"just read -- cat "${(e):-\$(touch x)}""#,
    "just read -- ls (touch x)",
    "just read -- cat $((1))",
];
const QUOTED_TEXT: [&str; 4] = [
    "just read -- cat '=(touch x)'",
    r#"just read -- cat "=(touch x)""#,
    "just read -- ls '*(e:cmd:)'",
    r#"just read -- cat "\${(e)x}""#,
];
const UNRESOLVED_RECIPES: [&str; 6] = [
    "./x/just lint",
    "/usr/local/bin/just lint",
    "just",
    "just jobs bootstrap",
    "just lint-rules bootstrap",
    "just --list bootstrap",
];

// DECISION: D202
// DECISION: D204
const DENIED_SPELLINGS: [&str; 12] = [
    "just write -- git rebase --exe=id main",
    "just write -- git rebase --exec id main",
    "just write -- git rebase -qxid main",
    "just write -- git commit --no-verify -qm x",
    "just write -- git commit --no-ver -qm x",
    "just write -- git commit -qnm x",
    "just read -- git grep --open-files-in-pa=cat x",
    "just read -- git grep -iOcat x",
    "just read -- rg --pr cat x",
    "just read -- rg --pre=cat x",
    "just read -- rg --hostname-bin=sh x",
    "just read -- find . -okdir id +",
];
const NEIGHBOR_FLAGS: [&str; 5] = [
    "just write -- git rebase --empty=drop main",
    "just write -- git rebase --no-ff main",
    "just read -- git grep --or x",
    "just read -- rg --pretty --pre-glob '*.gz' x",
    "just read -- find . -name x",
];

fn denied(root: &Path, command: &str) -> Value {
    let output = hook(root, "Bash", json!({"command": command}));
    let decision = &output["hookSpecificOutput"]["permissionDecision"];
    assert_eq!(decision, "deny", "{command}: {output}");
    output
}

// INVARIANT: I005
#[test]
fn shell_guard_rejects_bypass() {
    let (_temp, root) = project();
    for command in BYPASSES {
        denied(&root, command);
    }
}

#[test]
fn shell_guard_refuses_unreadable_check_entry() {
    let (_temp, root) = project();
    let output = denied(&root, r#"just write -- git commit -q -m "it's""#);
    let reason = reason(&output);
    for text in ["\"it's\"", "missing closing quote"] {
        assert!(reason.contains(text), "{reason}");
    }
}

fn rig_bootstrap(root: &Path) {
    let rig = fs::read_to_string(repository().join(".agentrig/commands.yaml")).unwrap();
    let catalog: serde_yaml_ng::Value = serde_yaml_ng::from_str(&rig).unwrap();
    let mut entry = serde_yaml_ng::Mapping::new();
    entry.insert("bootstrap".into(), catalog["bootstrap"].clone());
    let commands = serde_yaml_ng::to_string(&entry).unwrap();
    configure(root, "commands", &commands);
}

fn allowed(root: &Path, command: &str) {
    let output = hook(root, "Bash", json!({"command": command}));
    assert!(!output.to_string().contains("deny"), "{command}: {output}");
}

// DECISION: D197
#[test]
fn shell_guard_runs_required_steps_and_refuses_scratch() {
    let (_temp, root) = project();
    let scratch = denied(&root, "just scratch -- ls");
    assert!(reason(&scratch).contains("scratch"), "{scratch}");
    rig_bootstrap(&root);
    allowed(&root, "just bootstrap");
}

// DECISION: D218
#[test]
fn an_agent_installs_only_the_pinned_runtime_revision() {
    let (_temp, root) = project();
    rig_bootstrap(&root);
    configure(&root, "git", "install: bootstrap\n");
    allowed(&root, "just bootstrap");
    let skill = "propose-protected-change/SKILL.md";
    let refusals = ["just bootstrap HEAD", "just run bootstrap -- HEAD"];
    for command in refusals {
        let output = denied(&root, command);
        for named in [skill, "propose-pin", "! just approve"] {
            assert!(reason(&output).contains(named), "{command}: {output}");
        }
    }
}

#[test]
fn shell_base_programs_refuse_wrappers() {
    let (_temp, root) = project();
    for command in WRAPPERS {
        let output = denied(&root, command);
        let reason = reason(&output);
        for text in [" does not run ", "configured programs: "] {
            assert!(reason.contains(text), "{command}: {reason}");
        }
    }
}

#[test]
fn shell_catalog_and_recipe_listing() {
    let (_temp, root) = project();
    for command in LISTINGS {
        let output = hook(&root, "Bash", json!({"command": command}));
        assert!(!output.to_string().contains("deny"), "{command}: {output}");
    }
}

// DECISION: D201
#[test]
fn shell_guard_refuses_code_the_shell_runs_before_just_and_passes_it_quoted() {
    let (_temp, root) = project();
    for command in CODE_BEFORE_JUST {
        denied(&root, command);
    }
    for command in QUOTED_TEXT {
        let output = hook(&root, "Bash", json!({"command": command}));
        assert!(!output.to_string().contains("deny"), "{command}: {output}");
    }
}

#[test]
fn shell_guard_refuses_a_recipe_just_would_not_run_as_written() {
    let (_temp, root) = project();
    for command in UNRESOLVED_RECIPES {
        denied(&root, command);
    }
    let second_recipe = denied(&root, "just jobs bootstrap");
    let named = reason(&second_recipe).contains("takes no arguments");
    assert!(named, "{second_recipe}");
}

// DECISION: D202
#[test]
fn denied_flags_are_refused_in_every_spelling_their_program_reads() {
    let (_temp, root) = project();
    for command in DENIED_SPELLINGS {
        let output = denied(&root, command);
        let named = reason(&output).contains(" does not run ");
        assert!(named, "{command}: {output}");
    }
    for command in NEIGHBOR_FLAGS {
        let output = hook(&root, "Bash", json!({"command": command}));
        assert!(!output.to_string().contains("deny"), "{command}: {output}");
    }
}

#[test]
fn background_jobs_get_the_checks_of_the_command_they_start() {
    let (_temp, root) = project();
    let wrapper = denied(&root, "just read -- python3 tool.py");
    let started = denied(&root, "just job-start read -- python3 tool.py");
    assert_eq!(reason(&started), reason(&wrapper));
    let removal = json!({"command": "just job-start write -- rm Ledger/Plan/001.md"});
    let routed = hook(&root, "Bash", removal).to_string();
    let named = "edit-plan/SKILL.md for shell write of Ledger/Plan/001.md.";
    assert!(routed.contains(named), "{routed}");
}
