mod adapters;
mod assets;
mod doctor;
mod lint;
use super::config::{self, Check, CheckKind, Command, Config, Git, Hooks, Paths, Route};
use anyhow::{Result, ensure};
pub use doctor::run as doctor;
use std::{
    collections::BTreeMap,
    fs::{self, OpenOptions},
    io::Write,
    path::Path,
};

pub fn init(root: &Path, args: &[String]) -> Result<i32> {
    let mut options = BTreeMap::from([
        ("language", "python"),
        ("source", "src"),
        ("memory", "memory"),
        ("skills", ".worker/skills"),
        ("base", "main"),
        ("prefix", "feature/"),
    ]);
    ensure!(
        args.len().is_multiple_of(2),
        "init [--language python|rust] [--source PATH] [--memory PATH] [--skills PATH] [--base BRANCH] [--prefix PREFIX]"
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
    for key in ["source", "memory", "skills"] {
        config::relative(root, options[key])?;
    }
    let skill_root = options["skills"];
    let repair = format!("{skill_root}/repair/SKILL.md");
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
    let argv = if language == "python" {
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
    let mut routes = Vec::new();
    for (file, skill) in [
        ("Plan", "edit-plan"),
        ("Decisions", "edit-decisions"),
        ("Invariants", "edit-invariants"),
        ("State", "edit-state"),
    ] {
        routes.push(Route {
            include: vec![
                format!("{}/{file}.md", options["memory"]),
                format!("{}/{file}/**/*.md", options["memory"]),
            ],
            skill: format!("{skill_root}/{skill}/SKILL.md"),
        });
    }
    let config = Config {
        version: 1,
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
        commands,
        checks: vec![
            Check {
                id: "lint".into(),
                kind: CheckKind::Lint,
                command: None,
                include: vec!["**".into()],
                skill: repair.clone(),
                warning: false,
            },
            Check {
                id: "memory".into(),
                kind: CheckKind::Memory,
                command: None,
                include: vec!["**".into()],
                skill: repair.clone(),
                warning: false,
            },
            Check {
                id: "tests".into(),
                kind: CheckKind::Command,
                command: Some("test".into()),
                include: vec![format!("{}/**", options["source"])],
                skill: repair.clone(),
                warning: false,
            },
        ],
        hooks: Hooks {
            routes,
            reminder: Some(".worker/reminder.json".into()),
            discipline_skill: Some(format!("{skill_root}/complexity-discipline/SKILL.md")),
        },
        oracles: BTreeMap::new(),
    };
    let mut files = BTreeMap::<String, Vec<u8>>::new();
    files.insert(
        config::FILE.into(),
        toml::to_string_pretty(&config)?.into_bytes(),
    );
    for (name, source) in assets::skills() {
        files.insert(format!("{skill_root}/{name}/SKILL.md"), source.into_bytes());
    }
    for (name, source) in assets::memory() {
        files.insert(
            format!("{}/{name}.md", options["memory"]),
            source.into_bytes(),
        );
    }
    files.insert(
        ".worker/lint.toml".into(),
        lint::template(skill_root, options["source"])?.into_bytes(),
    );
    files.insert(
        ".worker/reminder.json".into(),
        include_bytes!("../../../assets/skills/complexity-discipline/context-reminder.json")
            .to_vec(),
    );
    files.insert(".worker/.gitignore".into(), b"runtime/\n".to_vec());
    files.insert(
        ".worker/bin/discipline-worker".into(),
        fs::read(std::env::current_exe()?)?,
    );
    files.insert("justfile".into(), justfile().into_bytes());
    files.insert(
        ".codex/config.toml".into(),
        adapters::CODEX_CONFIG.as_bytes().to_vec(),
    );
    files.insert(".codex/hooks.json".into(), adapters::registration()?);
    for (path, contents) in adapters::git_hooks() {
        files.insert(path.into(), contents.to_vec());
    }
    for path in files.keys() {
        let resolved = config::relative(root, path)?;
        for parent in resolved
            .ancestors()
            .skip(1)
            .take_while(|p| p.starts_with(root))
        {
            ensure!(
                !parent.exists() || parent.is_dir(),
                "init collision: {} is not a directory",
                parent.display()
            );
        }
        ensure!(
            !resolved.exists() && root.join(path).symlink_metadata().is_err(),
            "init collision: {path}; existing files were preserved"
        );
    }
    // Check existing Git hook ownership before creating any files.
    let git = root.join(".git").exists();
    if git {
        let existing =
            crate::util::git(root, &["config", "--get", "core.hooksPath"]).unwrap_or_default();
        ensure!(
            existing.is_empty(),
            "existing core.hooksPath={existing}; init will not replace it"
        );
    }
    // Validate generated files through the same loader before touching the consumer.
    // This also detects a generated file being another generated file's parent.
    let preview = tempfile::tempdir()?;
    for (path, contents) in &files {
        let path = config::relative(preview.path(), path)?;
        fs::create_dir_all(path.parent().unwrap())?;
        fs::write(path, contents)?;
    }
    config::Context::load(preview.path())?;
    for (path, contents) in files {
        let path = config::relative(root, &path)?;
        fs::create_dir_all(path.parent().unwrap())?;
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)?;
        file.write_all(&contents)?;
        let executable = path
            .components()
            .any(|c| c.as_os_str() == "bin" || c.as_os_str() == "hooks");
        if executable {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&path, fs::Permissions::from_mode(0o755))?;
        }
    }
    if git {
        crate::util::git(root, &["config", "core.hooksPath", ".worker/hooks"])?;
    }
    println!(
        "Initialized {language} scaffold with runtime {}. Run config-check and doctor; source files remain yours to create.",
        config::VERSION
    );
    Ok(0)
}
fn justfile() -> String {
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
    ] {
        source.push_str(&format!(
            "{name}{}:\n    @.worker/bin/discipline-worker {command} --root .{}\n\n",
            if args { " *args" } else { "" },
            if args { " \"$@\"" } else { "" }
        ));
    }
    source
}
