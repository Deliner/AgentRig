use crate::project_config::{
    self, Access, Arguments, Check, CheckKind, Command, Config, Flag, Git, Hooks, Paths, Program,
    Route, RouteEvent, Rust, Tools, View, Views,
};
use crate::project_memory::NORTH_STAR;
use crate::project_setup::rust::{ClippySettings, Requirement, build_rustflags, clippy_settings};
use crate::shared::calling_agent::Agent;
use crate::structural_lint::{CargoLayout, cargo_layout};
use anyhow::{Result, bail, ensure};
use std::{collections::BTreeMap, path::Path};
pub type Options<'a> = BTreeMap<&'a str, String>;
// DECISION: D040
// DECISION: D187
pub(super) const SOURCE: &str = "Project";
const MEMORY: &str = "Ledger";
// DECISION: D189
pub(super) const CACHE: &str = ".cache";
const ORIGIN: &str = "origin";
pub fn options<'a>(root: &Path, args: &'a [String]) -> Result<Options<'a>> {
    let mut options: Options<'_> = [
        ("language", "python"),
        ("skills", ".agentrig/skills"),
        ("service", ".agentrig"),
        ("base", "main"),
        ("prefix", "feature/"),
    ]
    .into_iter()
    .map(|(key, value)| (key, value.into()))
    .collect();
    ensure!(
        args.len().is_multiple_of(2),
        "init [--language python|rust] [--skills PATH] [--service PATH] [--base BRANCH] [--prefix PREFIX]"
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
    let language = options["language"].as_str();
    ensure!(
        ["python", "rust"].contains(&language),
        "init supports python or rust"
    );
    for key in ["skills", "service"] {
        project_config::relative(root, &options[key])?;
    }
    // DECISION: D188
    let skills = Path::new(&options["skills"]);
    let overlapping = Agent::ALL.into_iter().find(|agent| {
        let directory = Path::new(agent.skills());
        skills.starts_with(directory) || directory.starts_with(skills)
    });
    if let Some(agent) = overlapping {
        bail!(
            "init --skills {} overlaps {}, which init links to the skills",
            skills.display(),
            agent.skills()
        );
    }
    Ok(())
}
pub struct Scaffold {
    pub config: Config,
    pub rust: CargoLayout,
    pub clippy: Option<ClippySettings>,
}
pub fn scaffold(root: &Path, options: &Options<'_>) -> Result<Scaffold> {
    let python = options["language"] == "python";
    let (rust, clippy, settings) = if python {
        let rust = CargoLayout {
            roots: Vec::new(),
            ..CargoLayout::fallback(SOURCE)
        };
        (rust, None, Rust::default())
    } else {
        // DECISION: D174
        let toolchain = Requirement::inspect(root)?;
        ensure!(
            toolchain.met(),
            "init refused for a Rust project: {toolchain}; no files were written"
        );
        let rust = cargo_layout(root, SOURCE)?;
        let clippy = clippy_settings(root, &rust)?;
        // DECISION: D239
        let settings = Rust {
            manifest: Some(rust.manifest.clone()),
            rustflags: build_rustflags(root)?,
        };
        (rust, Some(clippy), settings)
    };
    let mut config = config(options, &rust, settings)?;
    config.git.remote = remote(root)?;
    Ok(Scaffold {
        config,
        rust,
        clippy,
    })
}
// DECISION: D189
fn remote(root: &Path) -> Result<Option<String>> {
    let Some(repository) = crate::shared::vcs::Repository::discover(root)? else {
        return Ok(None);
    };
    let remotes = String::from_utf8(repository.run(&["remote"])?)?;
    let origin = remotes.lines().any(|name| name == ORIGIN);
    Ok(origin.then(|| ORIGIN.to_owned()))
}
fn config(options: &Options<'_>, rust: &CargoLayout, settings: Rust) -> Result<Config> {
    let skill_root = &options["skills"];
    let service = &options["service"];
    let repair = format!("{skill_root}/repair/SKILL.md");
    let discipline = format!("{skill_root}/complexity-discipline");
    Ok(Config {
        version: 1,
        capabilities: project_config::Capabilities::default(),
        runtime: project_config::VERSION.into(),
        config_skill: repair.clone(),
        paths: Paths {
            service: service.clone(),
            sources: vec![format!("{SOURCE}/**")],
            memory: MEMORY.into(),
            skills: skill_root.into(),
            lint: format!("{service}/lint.yaml"),
            runtime: format!("{CACHE}/runtime"),
        },
        git: git(options),
        commands: commands(options, rust)?,
        checks: checks(options, skill_root),
        hooks: Hooks {
            routes: routes(options),
            reminder: Some(format!("{discipline}/context-reminder.json")),
            discipline_skill: Some(format!("{discipline}/SKILL.md")),
            protected: vec![format!("{MEMORY}/{NORTH_STAR}.md")],
        },
        oracles: BTreeMap::new(),
        tools: tools(service),
        rust: settings,
    })
}
// DECISION: D150
fn tools(service: &str) -> Tools {
    let installed = vec![format!("{service}/bin/**"), format!("{service}/hooks/*")];
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
        remote: None,
        pin: None,
        install: None,
        keep: Vec::new(),
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
    // DECISION: D202
    ("rg", &[], &["--pre", "--hostname-bin"]),
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
        // DECISION: D204
        &["--exec", "-x", "--no-verify", "-n"],
    ),
    ("mkdir", &[], &[]),
    ("cp", &[], &[]),
    ("mv", &[], &[]),
    ("rm", &[], &[]),
    ("touch", &[], &[]),
    ("chmod", &[], &[]),
    // DECISION: D198
    ("ln", &[], &[]),
];
fn programs(table: Programs) -> Result<BTreeMap<String, Program>> {
    let strings = |items: &[&str]| items.iter().map(|item| (*item).into()).collect();
    table
        .iter()
        .map(|(name, subcommands, deny)| {
            let deny = deny
                .iter()
                .map(|flag| Flag::try_from(String::from(*flag)))
                .collect::<Result<_>>()?;
            let program = Program {
                subcommands: strings(subcommands),
                deny,
                access: None,
            };
            Ok(((*name).into(), program))
        })
        .collect()
}
fn commands(options: &Options<'_>, rust: &CargoLayout) -> Result<BTreeMap<String, Command>> {
    let base = |purpose, access, table| -> Result<Command> {
        Ok(Command {
            programs: programs(table)?,
            ..command(purpose, Vec::new(), access)
        })
    };
    let read = base(
        "Inspect files and Git history with configured programs",
        Access::ReadOnly,
        READ,
    )?;
    let mut write = base(
        "Change files and Git state with configured programs",
        Access::Project,
        WRITE,
    )?;
    // DECISION: D203
    if let Some(git) = write.programs.get_mut("git") {
        git.access = Some(Access::Unconfined);
    }
    Ok([("read", read), ("write", write)]
        .into_iter()
        .chain(language_commands(options, rust))
        .map(|(id, command)| (id.into(), command))
        .collect())
}
fn language_commands(options: &Options<'_>, rust: &CargoLayout) -> Vec<(&'static str, Command)> {
    let rust_project = options["language"] == "rust";
    let purpose = if rust_project {
        "Run the project tests, optionally filtered by test name"
    } else {
        "Run the project tests, optionally filtered by test path"
    };
    // DECISION: D202
    let test = Command {
        accepts_args: Arguments::Operands,
        output: Some(test_views(options)),
        ..command(purpose, test_command(options, rust), Access::Unconfined)
    };
    let mut commands = vec![("test", test)];
    if rust_project {
        let check = |purpose, argv| Command {
            accepts_args: Arguments::None,
            ..command(purpose, argv, Access::Unconfined)
        };
        // DECISION: D189
        let format = ["cargo", "fmt", "--manifest-path", &rust.manifest, "--check"];
        commands.push((
            "rustfmt",
            check("Check Rust formatting", format.map(String::from).into()),
        ));
    }
    commands
}
fn command(purpose: &str, argv: Vec<String>, access: Access) -> Command {
    Command {
        purpose: purpose.into(),
        doc: None,
        lifetime: project_config::Lifetime::Task,
        argv,
        cwd: ".".into(),
        accepts_args: Arguments::Any,
        access,
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
fn test_command(options: &Options<'_>, rust: &CargoLayout) -> Vec<String> {
    let python = options["language"] == "python";
    if python {
        vec![
            "python3".into(),
            "-m".into(),
            "pytest".into(),
            SOURCE.into(),
        ]
    } else {
        vec![
            "cargo".into(),
            "test".into(),
            "--manifest-path".into(),
            rust.manifest.clone(),
        ]
    }
}
// DECISION: D110
fn routes(options: &Options<'_>) -> Vec<Route> {
    let skill_root = &options["skills"];
    let mut routes = vec![Route {
        include: vec![format!("{SOURCE}/**")],
        skill: format!("{skill_root}/refactor-large-directory/SKILL.md"),
        events: vec![RouteEvent::Edit],
        block: true,
    }];
    routes.extend(memory_routes(options));
    routes.push(Route {
        include: memory_files("State"),
        skill: format!("{skill_root}/edit-state/SKILL.md"),
        events: vec![RouteEvent::Check],
        block: true,
    });
    // DECISION: D186
    routes.push(Route {
        include: memory_files("Decisions"),
        skill: format!("{skill_root}/norm-or-choice/SKILL.md"),
        events: vec![RouteEvent::Edit],
        block: true,
    });
    routes.push(Route {
        include: vec![format!("{SOURCE}/**")],
        skill: format!("{skill_root}/fail-closed-enforcement/SKILL.md"),
        events: vec![RouteEvent::Edit],
        block: false,
    });
    routes
}
fn memory_routes(options: &Options<'_>) -> Vec<Route> {
    let skill_root = &options["skills"];
    [
        ("Plan", "edit-plan"),
        ("Archive/Plan", "edit-plan"),
        ("Backlog", "edit-backlog"),
        ("Decisions", "edit-decisions"),
        ("Invariants", "edit-invariants"),
        ("State", "edit-state"),
    ]
    .into_iter()
    .map(|(file, skill)| Route {
        include: memory_files(file),
        skill: format!("{skill_root}/{skill}/SKILL.md"),
        events: vec![RouteEvent::Edit, RouteEvent::Shell],
        block: false,
    })
    .collect()
}
fn memory_files(file: &str) -> Vec<String> {
    vec![
        format!("{MEMORY}/{file}.md"),
        format!("{MEMORY}/{file}/**/*.md"),
    ]
}
fn checks(options: &Options<'_>, skill_root: &str) -> Vec<Check> {
    let skill = |name| format!("{skill_root}/{name}/SKILL.md");
    let run = |id, command: &str| Check {
        command: Some(command.into()),
        include: vec![format!("{SOURCE}/**")],
        ..check(id, CheckKind::Command, skill("repair"))
    };
    let mut checks = vec![
        check("lint", CheckKind::Lint, skill("repair")),
        check("memory", CheckKind::Memory, skill("repair")),
        check("tools", CheckKind::Tools, skill("develop-tools")),
    ];
    let rust_project = options["language"] == "rust";
    // DECISION: D189
    if rust_project {
        checks.push(run("rustfmt", "rustfmt"));
        checks.push(run("rust-test", "test"));
    } else {
        checks.push(run("tests", "test"));
    }
    checks
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
