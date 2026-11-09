// DECISION: D124
// DECISION: D138
use crate::change_delivery::gate::{AffectedMap, files, included};
use crate::project_config::{Check, Context};
use crate::shared::diagnostics::{Guidance, rerun};
use crate::shared::{console, fs};
use anyhow::Result;
use std::path::{Component, Path, PathBuf};

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
}

pub(super) fn report(context: &Context) -> Result<bool> {
    let files = files(context)?;
    let rerun = rerun(&context.root, &["config-check".into()]);
    let mut found = false;
    for check in &context.config.checks {
        let Some(mapped) = mapped(context, check)? else {
            continue;
        };
        let extension = Some(std::ffi::OsStr::new(mapped.runner.source()));
        let sources = included(check, &files)?.filter(|path| path.extension() == extension);
        for path in sources {
            let Some(message) = finding(&mapped, path)? else {
                continue;
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

fn mapped<'a>(context: &'a Context, check: &'a Check) -> Result<Option<Mapped<'a>>> {
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
    Ok(Some(Mapped {
        root: &context.root,
        check,
        runner,
        cwd: &command.cwd,
        map: AffectedMap::new(check)?,
    }))
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
        Runner::Cargo => module(mapped.root, path).is_none_or(|module| {
            targets(mapped)
                .any(|target| module.contains(target.as_str()) || target.starts_with(&module))
        }),
    }
}

fn normalized(path: &Path) -> PathBuf {
    let mut result = PathBuf::new();
    for component in path.components() {
        match component {
            Component::ParentDir => {
                result.pop();
            }
            Component::Normal(part) => result.push(part),
            _ => {}
        }
    }
    result
}

fn module(root: &Path, path: &Path) -> Option<String> {
    let crate_root = |entry: &str| {
        path.ancestors()
            .skip(1)
            .find(|directory| root.join(directory).join(entry).is_file())
    };
    let directory = crate_root("lib.rs").or_else(|| crate_root("main.rs"))?;
    let relative = path.strip_prefix(directory).ok()?.with_extension("");
    let mut parts: Vec<_> = relative.iter().filter_map(|part| part.to_str()).collect();
    let entry = matches!(parts.as_slice(), ["lib" | "main"]) || parts.last() == Some(&"mod");
    if entry {
        parts.pop();
    }
    Some(parts.iter().map(|part| format!("{part}::")).collect())
}
