use super::super::config::{self, Check, CheckKind, Command, Config, Hooks, Paths, Route, Vcs};
use anyhow::{Result, ensure};
use std::{collections::BTreeMap, path::Path};
pub(super) type Options<'a> = BTreeMap<&'a str, String>;
pub(super) fn options<'a>(root: &Path, args: &'a [String]) -> Result<Options<'a>> {
    let mut options: Options<'_> = [
        ("language", "python"),
        ("review", "false"),
        ("source", "src"),
        ("memory", "memory"),
        ("skills", ".agentrig/skills"),
        ("service", ".agentrig"),
        ("base", "main"),
        ("prefix", "feature/"),
        ("vcs", "git"),
        ("frontend", "codex"),
    ]
    .into_iter()
    .map(|(key, value)| (key, value.into()))
    .collect();
    ensure!(
        args.len().is_multiple_of(2),
        "init [--language python|rust] [--vcs git|mercurial] [--frontend codex|claude-code] [--source PATH] [--memory PATH] [--skills PATH] [--service PATH] [--base BRANCH] [--prefix PREFIX] [--review true|false]"
    );
    for pair in args.as_chunks::<2>().0 {
        let key = pair[0].strip_prefix("--").unwrap_or("");
        ensure!(options.contains_key(key), "unknown init option {}", pair[0]);
        options.insert(key, pair[1].clone());
    }
    let default_skills = !args
        .as_chunks::<2>()
        .0
        .iter()
        .any(|pair| pair[0] == "--skills");
    if default_skills {
        options.insert("skills", format!("{}/skills", options["service"]));
    }
    validate_options(root, &options)?;
    Ok(options)
}
fn validate_options(root: &Path, options: &Options<'_>) -> Result<()> {
    ensure!(
        ["codex", "claude-code"].contains(&options["frontend"].as_str()),
        "init --frontend expects codex or claude-code"
    );
    ensure!(
        ["git", "mercurial"].contains(&options["vcs"].as_str()),
        "init --vcs expects git or mercurial"
    );
    let language = options["language"].as_str();
    ensure!(
        ["python", "rust"].contains(&language),
        "init supports python or rust"
    );
    ensure!(
        ["true", "false"].contains(&options["review"].as_str()),
        "init --review expects true or false"
    );
    for key in ["source", "memory", "skills", "service"] {
        config::relative(root, &options[key])?;
    }
    Ok(())
}
pub(super) fn config(options: &Options<'_>) -> Config {
    let skill_root = &options["skills"];
    let service = &options["service"];
    let repair = format!("{skill_root}/repair/SKILL.md");
    Config {
        frontend: match options["frontend"].as_str() {
            "claude-code" => config::Frontend::ClaudeCode,
            _ => config::Frontend::Codex,
        },
        environment: Default::default(),
        version: 1,
        processes: Default::default(),
        capabilities: config::Capabilities {
            lint: true,
            delegation: None,
            review: (options["review"] == "true").then(|| config::Resource {
                config: format!("{service}/review/config/review.yaml"),
            }),
        },
        runtime: config::VERSION.into(),
        config_skill: repair.clone(),
        paths: Paths {
            service: service.clone(),
            sources: vec![format!("{}/**", options["source"])],
            memory: options["memory"].clone(),
            skills: skill_root.into(),
            lint: format!("{service}/lint.yaml"),
            runtime: format!("{service}/runtime"),
        },
        vcs: vcs(options),
        commands: commands(options),
        checks: checks(&options["source"], &repair),
        hooks: Hooks {
            routes: routes(&options["memory"], skill_root),
            reminder: Some(format!("{service}/reminder.json")),
            discipline_skill: Some(format!("{skill_root}/complexity-discipline/SKILL.md")),
        },
        oracles: BTreeMap::new(),
    }
}
fn vcs(options: &Options<'_>) -> Vcs {
    Vcs {
        backend: match options["vcs"].as_str() {
            "mercurial" => review_runner::vcs::Kind::Mercurial,
            _ => review_runner::vcs::Kind::Git,
        }
        .into(),
        base: options["base"].clone(),
        prefix: options["prefix"].clone(),
    }
}

fn commands(options: &Options<'_>) -> BTreeMap<String, Command> {
    let mut commands = BTreeMap::new();
    for (id, read_only) in [("read", true), ("write", false)] {
        commands.insert(
            id.into(),
            Command {
                lifetime: Default::default(),
                argv: Vec::new(),
                cwd: ".".into(),
                accepts_args: true,
                read_only,
            },
        );
    }
    let argv = test_command(options);
    commands.insert(
        "test".into(),
        Command {
            lifetime: Default::default(),
            argv,
            cwd: ".".into(),
            accepts_args: true,
            read_only: false,
        },
    );
    commands
}
fn test_command(options: &Options<'_>) -> Vec<String> {
    let python = options["language"] == "python";
    if python {
        vec![
            "python3".into(),
            "-m".into(),
            "pytest".into(),
            options["source"].clone(),
        ]
    } else {
        vec![
            "cargo".into(),
            "test".into(),
            "--manifest-path".into(),
            format!("{}/Cargo.toml", options["source"]),
        ]
    }
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
pub(super) fn justfile(config: &Config) -> String {
    let mut source = "set positional-arguments := true\n\n".to_owned();
    let path = format!("./{}", config.paths.service_path("bin/agentrig"));
    let binary = shell_words::quote(&path);
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
        ("setup", "setup", true),
        ("lint", "lint", true),
        ("lint-config-check", "lint-config-check", true),
        ("lint-rule", "lint-rule", true),
        ("lint-explain", "lint-explain", true),
        ("jobs", "jobs", false),
        ("job-status", "job-status", true),
        ("job-logs", "job-logs", true),
        ("job-start", "job-start", true),
        ("job-stop", "job-stop", true),
        ("job-cleanup", "job-cleanup", true),
        ("review", "review", true),
        ("delegate", "delegate", true),
    ] {
        source.push_str(&format!(
            "# What: invoke {command}; Why: use the installed native runtime.\n{name}{}:\n    @{binary} {command} --root .{}\n\n",
            if args { " *args" } else { "" },
            if args { " \"$@\"" } else { "" }
        ));
    }
    source
}
