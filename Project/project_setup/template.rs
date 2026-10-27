use crate::project_config::{
    self, Check, CheckKind, Command, Config, Git, Hooks, Paths, Program, Route, RouteEvent, Tools,
    View, Views,
};
use anyhow::{Result, ensure};
use std::{collections::BTreeMap, path::Path};
pub(super) type Options<'a> = BTreeMap<&'a str, String>;
// DECISION: D040
pub(super) fn options<'a>(root: &Path, args: &'a [String]) -> Result<Options<'a>> {
    let mut options: Options<'_> = [
        ("language", "python"),
        ("source", "Project"),
        ("memory", "Ledger"),
        ("skills", ".agentrig/skills"),
        ("service", ".agentrig"),
        ("base", "main"),
        ("prefix", "feature/"),
        ("frontend", "codex"),
    ]
    .into_iter()
    .map(|(key, value)| (key, value.into()))
    .collect();
    ensure!(
        args.len().is_multiple_of(2),
        "init [--language python|rust] [--frontend codex|claude-code] [--source PATH] [--memory PATH] [--skills PATH] [--service PATH] [--base BRANCH] [--prefix PREFIX]"
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
    let language = options["language"].as_str();
    ensure!(
        ["python", "rust"].contains(&language),
        "init supports python or rust"
    );
    for key in ["source", "memory", "skills", "service"] {
        project_config::relative(root, &options[key])?;
    }
    Ok(())
}
pub(super) fn config(options: &Options<'_>) -> Config {
    let skill_root = &options["skills"];
    let service = &options["service"];
    let repair = format!("{skill_root}/repair/SKILL.md");
    Config {
        frontend: match options["frontend"].as_str() {
            "claude-code" => project_config::Frontend::ClaudeCode,
            _ => project_config::Frontend::Codex,
        },
        version: 1,
        capabilities: Default::default(),
        runtime: project_config::VERSION.into(),
        config_skill: repair.clone(),
        paths: Paths {
            service: service.clone(),
            sources: vec![format!("{}/**", options["source"])],
            memory: options["memory"].clone(),
            skills: skill_root.into(),
            lint: format!("{service}/lint.yaml"),
            runtime: format!("{service}/runtime"),
        },
        git: git(options),
        commands: commands(options),
        checks: checks(&options["source"], skill_root),
        hooks: Hooks {
            routes: routes(&options["memory"], skill_root),
            reminder: Some(format!("{service}/reminder.json")),
            discipline_skill: Some(format!("{skill_root}/complexity-discipline/SKILL.md")),
        },
        oracles: BTreeMap::new(),
        tools: tools(service),
    }
}
fn tools(service: &str) -> Tools {
    let installed = vec![format!("{service}/bin/*"), format!("{service}/hooks/*")];
    Tools {
        recognize: Vec::new(),
        locations: installed.clone(),
        entries: installed,
        removed: BTreeMap::new(),
    }
}
fn git(options: &Options<'_>) -> Git {
    Git {
        base: options["base"].clone(),
        prefix: options["prefix"].clone(),
    }
}

type Programs = &'static [(
    &'static str,
    &'static [&'static str],
    &'static [&'static str],
)];
const READ: Programs = &[
    (
        "git",
        &[
            "status",
            "diff",
            "log",
            "show",
            "rev-parse",
            "branch",
            "ls-files",
            "grep",
            "blame",
            "merge-base",
            "describe",
            "cat-file",
            "ls-tree",
        ],
        &["-O", "--open-files-in-pager"],
    ),
    ("ls", &[], &[]),
    ("cat", &[], &[]),
    ("head", &[], &[]),
    ("tail", &[], &[]),
    ("wc", &[], &[]),
    ("grep", &[], &[]),
    ("rg", &[], &["--pre"]),
    ("sed", &[], &[]),
    ("diff", &[], &[]),
    ("stat", &[], &[]),
    ("find", &[], &["-exec", "-ok"]),
];
const WRITE: Programs = &[
    (
        "git",
        &[
            "add",
            "commit",
            "rm",
            "mv",
            "restore",
            "switch",
            "checkout",
            "stash",
            "cherry-pick",
            "rebase",
            "branch",
        ],
        &["--exec", "-x"],
    ),
    ("mkdir", &[], &[]),
    ("cp", &[], &[]),
    ("mv", &[], &[]),
    ("rm", &[], &[]),
    ("touch", &[], &[]),
    ("chmod", &[], &[]),
];
fn programs(table: Programs) -> BTreeMap<String, Program> {
    let strings = |items: &[&str]| items.iter().map(|item| (*item).into()).collect();
    table
        .iter()
        .map(|(name, subcommands, deny)| {
            let program = Program {
                subcommands: strings(subcommands),
                deny: strings(deny),
            };
            ((*name).into(), program)
        })
        .collect()
}
fn commands(options: &Options<'_>) -> BTreeMap<String, Command> {
    let base = |purpose, read_only, table| Command {
        programs: programs(table),
        ..command(purpose, Vec::new(), read_only)
    };
    let test = Command {
        output: Some(test_views(options)),
        ..command("Run the project tests", test_command(options), false)
    };
    [
        (
            "read",
            base(
                "Inspect files and Git history with configured programs",
                true,
                READ,
            ),
        ),
        (
            "write",
            base(
                "Change files and Git state with configured programs",
                false,
                WRITE,
            ),
        ),
        ("test", test),
    ]
    .into_iter()
    .map(|(id, command)| (id.into(), command))
    .collect()
}
fn command(purpose: &str, argv: Vec<String>, read_only: bool) -> Command {
    Command {
        purpose: purpose.into(),
        doc: None,
        lifetime: Default::default(),
        argv,
        cwd: ".".into(),
        accepts_args: true,
        read_only,
        programs: BTreeMap::new(),
        output: None,
    }
}
fn test_views(options: &Options<'_>) -> Views {
    let view = |matching: &[&str], tail| View {
        matching: matching.iter().map(|part| (*part).into()).collect(),
        tail,
    };
    let python = options["language"] == "python";
    if python {
        Views {
            success: view(&[], 1),
            failure: view(&["FAILED", "ERROR"], 1),
        }
    } else {
        Views {
            success: view(&["test result:"], 0),
            failure: view(&["FAILED", "panicked", "error", "test result:"], 0),
        }
    }
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
        ("Archive/Plan", "edit-plan"),
        ("Backlog", "edit-backlog"),
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
            events: vec![RouteEvent::Edit, RouteEvent::Shell],
            block: false,
        });
    }
    routes.push(Route {
        include: vec![
            format!("{memory}/State.md"),
            format!("{memory}/State/**/*.md"),
        ],
        skill: format!("{skill_root}/edit-state/SKILL.md"),
        events: vec![RouteEvent::Check],
        block: true,
    });
    routes
}
fn checks(source: &str, skill_root: &str) -> Vec<Check> {
    let skill = |name| format!("{skill_root}/{name}/SKILL.md");
    let tests = Check {
        command: Some("test".into()),
        include: vec![format!("{source}/**")],
        ..check("tests", CheckKind::Command, skill("repair"))
    };
    vec![
        check("lint", CheckKind::Lint, skill("repair")),
        check("memory", CheckKind::Memory, skill("repair")),
        tests,
        check("tools", CheckKind::Tools, skill("develop-tools")),
    ]
}
fn check(id: &str, kind: CheckKind, skill: String) -> Check {
    Check {
        id: id.into(),
        kind,
        command: None,
        include: vec!["**".into()],
        skill,
        warning: false,
        affected: Vec::new(),
    }
}
type Recipe = (&'static str, &'static str, bool);
// DECISION: D039
const RECIPES: [(&str, &[Recipe]); 5] = [
    (
        "commands",
        &[
            ("list", "commands", false),
            ("run", "run", true),
            ("scratch", "scratch", true),
        ],
    ),
    (
        "checks",
        &[
            ("check", "check", true),
            ("config-check", "config-check", false),
        ],
    ),
    (
        "delivery",
        &[
            ("resume", "resume", false),
            ("report", "report", false),
            ("feature-start", "feature-start", true),
            ("feature-merge", "feature-merge", false),
        ],
    ),
    (
        "lint",
        &[
            ("lint", "lint", true),
            ("lint-config-check", "lint-config-check", true),
            ("lint-rule", "lint-rule", true),
            ("lint-explain", "lint-explain", true),
        ],
    ),
    (
        "jobs",
        &[
            ("jobs", "jobs", false),
            ("job-status", "job-status", true),
            ("job-logs", "job-logs", true),
            ("job-start", "job-start", true),
            ("job-stop", "job-stop", true),
            ("job-cleanup", "job-cleanup", true),
        ],
    ),
];
pub(super) fn justfiles(config: &Config) -> Vec<(String, String)> {
    let mut root = "set positional-arguments := true\n\n".to_owned();
    let path = format!("./{}", config.paths.service_path("bin/agentrig"));
    let binary = shell_words::quote(&path);
    let mut files = Vec::new();
    for (group, recipes) in RECIPES {
        let file = format!("just/{group}.just");
        root.push_str(&format!("import '{file}'\n"));
        let source: Vec<String> = recipes
            .iter()
            .map(|&(name, command, args)| {
                format!(
                    "# What: invoke {command}; Why: use the installed native runtime.\n{name}{}:\n    @{binary} {command} --root .{}\n",
                    if args { " *args" } else { "" },
                    if args { " \"$@\"" } else { "" }
                )
            })
            .collect();
        files.push((file, source.join("\n")));
    }
    files.push(("justfile".into(), root));
    files
}
