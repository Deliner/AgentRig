// DECISION: D124
// DECISION: D138
use crate::change_delivery::gate::{AffectedMap, files, included};
use crate::project_config::{Check, Context};
use crate::shared::diagnostics::{Guidance, rerun};
use crate::shared::{console, fs_with_context as fs};
use crate::structural_lint::architecture::{extraction, resolution::rust::Rust};
use anyhow::Result;
use std::collections::BTreeMap;
use std::path::{Component, Path, PathBuf};

type Modules = BTreeMap<PathBuf, Vec<Vec<String>>>;

const UNSELECTED: &str = "no affected group targets this test, so the gate never selects it; add it or its directory to the targets of the group whose changes it verifies";

#[derive(Clone, Copy)]
enum Runner {
    Cargo,
    Pytest,
}

impl Runner {
    fn infer(argv: &[String]) -> Option<Self> {
        let named =
            |arg: &String, name: &str| Path::new(arg).file_name().is_some_and(|file| file == name);
        let pytest = argv.iter().any(|arg| named(arg, "pytest"));
        if pytest {
            return Some(Self::Pytest);
        }
        let cargo = argv.iter().position(|arg| named(arg, "cargo"))?;
        let testing = argv.iter().skip(cargo + 1).any(|arg| arg == "test");
        testing.then_some(Self::Cargo)
    }
    fn source(self) -> &'static str {
        match self {
            Self::Cargo => "rs",
            Self::Pytest => "py",
        }
    }
    fn everything(self) -> &'static str {
        match self {
            Self::Cargo => "::",
            Self::Pytest => ".",
        }
    }
}

struct Mapped<'a> {
    root: &'a Path,
    check: &'a Check,
    runner: Runner,
    cwd: &'a str,
    map: AffectedMap<'a>,
    modules: Modules,
}

pub(super) fn report(context: &Context) -> Result<bool> {
    let files = files(context)?;
    let rerun = rerun(&context.root, &["config-check".into()]);
    let mut found = false;
    for check in &context.config.checks {
        let Some(mapped) = mapped(context, check, &files)? else {
            continue;
        };
        let extension = Some(std::ffi::OsStr::new(mapped.runner.source()));
        let sources = included(check, &files)?.filter(|path| path.extension() == extension);
        'sources: for path in sources {
            let Some(message) = finding(&mapped, path)? else {
                continue 'sources;
            };
            found = true;
            console::stderr(Guidance {
                level: "ERROR",
                id: &check.id,
                location: &path.display().to_string(),
                message: &message,
                skill: &context.config.config_skill,
                rerun: &rerun,
            });
        }
    }
    Ok(found)
}

fn mapped<'a>(
    context: &'a Context,
    check: &'a Check,
    files: &[PathBuf],
) -> Result<Option<Mapped<'a>>> {
    let command = check
        .command
        .as_ref()
        .and_then(|name| context.config.commands.get(name));
    let Some(command) = command else {
        return Ok(None);
    };
    let Some(runner) = Runner::infer(&command.argv) else {
        return Ok(None);
    };
    let unmapped = check.affected.is_empty();
    if unmapped {
        return Ok(None);
    }
    let modules = match runner {
        Runner::Cargo => modules(&context.root, files, &command.argv, &command.cwd)?,
        Runner::Pytest => Modules::new(),
    };
    Ok(Some(Mapped {
        root: &context.root,
        check,
        runner,
        cwd: &command.cwd,
        map: AffectedMap::new(check)?,
        modules,
    }))
}

// DECISION: D144
fn modules(root: &Path, files: &[PathBuf], argv: &[String], cwd: &str) -> Result<Modules> {
    let manifest = manifest(argv, cwd);
    let mut modules = Modules::new();
    let absent = !root.join(&manifest).is_file();
    if absent {
        return Ok(modules);
    }
    let directory = manifest.parent().unwrap_or(Path::new(""));
    let mut sources = BTreeMap::new();
    let rust = |path: &&PathBuf| {
        path.starts_with(directory) && path.extension().is_some_and(|extension| extension == "rs")
    };
    for path in files.iter().filter(rust) {
        let text = fs::read_to_string(root.join(path))?;
        sources.insert(path.clone(), extraction::extract(path, &text)?);
    }
    let layout = super::rust::cargo::manifest_layout(root, &manifest.to_string_lossy())?;
    for crate_root in layout.roots {
        for (file, bases) in Rust::modules(Path::new(&crate_root), &sources) {
            modules.entry(file).or_default().extend(bases);
        }
    }
    Ok(modules)
}

fn manifest(argv: &[String], cwd: &str) -> PathBuf {
    let flag = argv.iter().position(|arg| arg == "--manifest-path");
    let separate = flag
        .and_then(|index| argv.get(index + 1))
        .map(String::as_str);
    let joined = argv
        .iter()
        .find_map(|arg| arg.strip_prefix("--manifest-path="));
    let path = separate.or(joined).unwrap_or("Cargo.toml");
    normalized(&Path::new(cwd).join(path))
}

fn finding(mapped: &Mapped<'_>, path: &Path) -> Result<Option<String>> {
    let silent = mapped
        .map
        .targets(path)
        .is_some_and(|targets| targets.is_empty());
    if silent {
        return Ok(Some(format!(
            "only affected groups without targets match this file, so changing it selects no tests; add it to a group whose targets exercise it, or map it to '{}' (every test) when its consumers are unknown",
            mapped.runner.everything()
        )));
    }
    let unselected = test(mapped, path)? && !selected(mapped, path);
    Ok(unselected.then(|| UNSELECTED.to_owned()))
}

fn test(mapped: &Mapped<'_>, path: &Path) -> Result<bool> {
    Ok(match mapped.runner {
        Runner::Pytest => path
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.starts_with("test_")),
        Runner::Cargo => fs::read_to_string(mapped.root.join(path))?.contains("#[test]"),
    })
}

fn targets<'a>(mapped: &'a Mapped<'_>) -> impl Iterator<Item = &'a String> {
    mapped
        .check
        .affected
        .iter()
        .flat_map(|group| &group.targets)
}

fn selected(mapped: &Mapped<'_>, path: &Path) -> bool {
    match mapped.runner {
        Runner::Pytest => targets(mapped).any(|target| {
            let file = target.split("::").next().unwrap_or_default();
            path.starts_with(normalized(&Path::new(mapped.cwd).join(file)))
        }),
        Runner::Cargo => mapped.modules.get(path).is_none_or(|modules| {
            modules
                .iter()
                .any(|module| targets(mapped).any(|target| selects(module, target)))
        }),
    }
}

// DECISION: D144
fn selects(module: &[String], target: &str) -> bool {
    let suffix = |start: usize| -> String {
        module
            .iter()
            .skip(start)
            .map(|part| format!("{part}::"))
            .collect()
    };
    let crate_root = module.is_empty();
    let inside = suffix(0).contains(target);
    let continued = (0..module.len()).any(|start| target.starts_with(&suffix(start)));
    crate_root || inside || continued
}

fn normalized(path: &Path) -> PathBuf {
    let mut result = PathBuf::new();
    for component in path.components() {
        match component {
            Component::ParentDir => {
                result.pop();
            }
            Component::Normal(part) => result.push(part),
            Component::Prefix(_) | Component::RootDir | Component::CurDir => {}
        }
    }
    result
}
