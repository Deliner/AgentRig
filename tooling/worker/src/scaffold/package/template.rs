use super::super::config::{self, Check, CheckKind, Command, Config, Git, Hooks, Paths, Route};
use anyhow::{Result, ensure};
use std::{collections::BTreeMap, path::Path};
pub(super) type Options<'a> = BTreeMap<&'a str, &'a str>;
pub(super) fn options<'a>(root: &Path, args: &'a [String]) -> Result<Options<'a>> {
    let mut options = BTreeMap::from([
        ("language", "python"),
        ("review", "false"),
        ("source", "src"),
        ("memory", "memory"),
        ("skills", ".worker/skills"),
        ("base", "main"),
        ("prefix", "feature/"),
    ]);
    ensure!(
        args.len().is_multiple_of(2),
        "init [--language python|rust] [--source PATH] [--memory PATH] [--skills PATH] [--base BRANCH] [--prefix PREFIX] [--review true|false]"
    );
    for pair in args.as_chunks::<2>().0 {
        let key = pair[0].strip_prefix("--").unwrap_or("");
        ensure!(options.contains_key(key), "unknown init option {}", pair[0]);
        options.insert(key, &pair[1]);
    }
    let language = options["language"];
    ensure!(
        ["python", "rust"].contains(&language),
        "init supports python or rust"
    );
    ensure!(
        ["true", "false"].contains(&options["review"]),
        "init --review expects true or false"
    );
    for key in ["source", "memory", "skills"] {
        config::relative(root, options[key])?;
    }
    Ok(options)
}
pub(super) fn config(options: &Options<'_>) -> Config {
    let skill_root = options["skills"];
    let repair = format!("{skill_root}/repair/SKILL.md");
    Config {
        version: 1,
        capabilities: config::Capabilities {
            lint: true,
            review: (options["review"] == "true").then(|| config::Review {
                config: super::review::CONFIG.into(),
            }),
        },
        runtime: config::VERSION.into(),
        config_skill: repair.clone(),
        paths: Paths {
            sources: vec![format!("{}/**", options["source"])],
            memory: options["memory"].into(),
            skills: skill_root.into(),
            lint: ".worker/lint.toml".into(),
            runtime: ".worker/runtime".into(),
        },
        git: Git {
            base: options["base"].into(),
            prefix: options["prefix"].into(),
        },
        commands: commands(options),
        checks: checks(options["source"], &repair),
        hooks: Hooks {
            routes: routes(options["memory"], skill_root),
            reminder: Some(".worker/reminder.json".into()),
            discipline_skill: Some(format!("{skill_root}/complexity-discipline/SKILL.md")),
        },
        oracles: BTreeMap::new(),
    }
}
fn commands(options: &Options<'_>) -> BTreeMap<String, Command> {
    let mut commands = BTreeMap::new();
    for (id, read_only) in [("read", true), ("write", false)] {
        commands.insert(
            id.into(),
            Command {
                argv: Vec::new(),
                cwd: ".".into(),
                accepts_args: true,
                read_only,
            },
        );
    }
    let python = options["language"] == "python";
    let argv = if python {
        vec![
            "python3".into(),
            "-m".into(),
            "pytest".into(),
            options["source"].into(),
        ]
    } else {
        vec![
            "cargo".into(),
            "test".into(),
            "--manifest-path".into(),
            format!("{}/Cargo.toml", options["source"]),
        ]
    };
    commands.insert(
        "test".into(),
        Command {
            argv,
            cwd: ".".into(),
            accepts_args: true,
            read_only: false,
        },
    );
    commands
}
fn routes(memory: &str, skill_root: &str) -> Vec<Route> {
    let mut routes = Vec::new();
    for (file, skill) in [
        ("Plan", "edit-plan"),
        ("Decisions", "edit-decisions"),
        ("Invariants", "edit-invariants"),
        ("State", "edit-state"),
    ] {
        routes.push(Route {
            include: vec![
                format!("{}/{file}.md", memory),
                format!("{}/{file}/**/*.md", memory),
            ],
            skill: format!("{skill_root}/{skill}/SKILL.md"),
        });
    }
    routes
}
fn checks(source: &str, repair: &str) -> Vec<Check> {
    vec![
        Check {
            id: "lint".into(),
            kind: CheckKind::Lint,
            command: None,
            include: vec!["**".into()],
            skill: repair.to_owned(),
            warning: false,
        },
        Check {
            id: "memory".into(),
            kind: CheckKind::Memory,
            command: None,
            include: vec!["**".into()],
            skill: repair.to_owned(),
            warning: false,
        },
        Check {
            id: "tests".into(),
            kind: CheckKind::Command,
            command: Some("test".into()),
            include: vec![format!("{source}/**")],
            skill: repair.to_owned(),
            warning: false,
        },
    ]
}
pub(super) fn justfile() -> String {
    let mut source = "set positional-arguments := true\n\n".to_owned();
    for (name, command, args) in [
        ("list", "commands", false),
        ("run", "run", true),
        ("check", "check", true),
        ("resume", "resume", false),
        ("config-check", "config-check", false),
        ("report", "report", false),
        ("feature-start", "feature-start", true),
        ("feature-merge", "feature-merge", false),
        ("upgrade", "upgrade", true),
        ("setup", "setup", false),
        ("lint", "lint", true),
        ("lint-config-check", "lint-config-check", true),
        ("review", "review", true),
    ] {
        source.push_str(&format!(
            "# What: invoke {command}; Why: use the installed native runtime.\n{name}{}:\n    @.worker/bin/discipline-worker {command} --root .{}\n\n",
            if args { " *args" } else { "" },
            if args { " \"$@\"" } else { "" }
        ));
    }
    source
}
