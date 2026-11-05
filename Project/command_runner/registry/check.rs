use crate::project_config::{Check, Context, Removed, module};
use crate::shared::diagnostics::lint::{Diagnostic, Level, print_diagnostics};
use crate::shared::fs;
use crate::shared::paths::patterns::globs;
use anyhow::Result;
use std::path::Path;

struct Finding {
    path: String,
    message: String,
}
type Findings = Vec<Finding>;
pub fn run(context: &Context, check: &Check, rerun: &str) -> Result<i32> {
    let files = crate::shared::vcs::Repository::new(&context.root).working_files()?;
    let tools = super::tools(context, &files)?;
    let mut findings = Findings::new();
    placement(context, &tools, &mut findings)?;
    catalog(context, &mut findings);
    removed(context, &files, &mut findings);
    let diagnostics: Vec<_> = findings
        .into_iter()
        .map(|finding| Diagnostic {
            rule: check.id.clone(),
            path: finding.path,
            level: Level::Error,
            actual: None,
            limit: None,
            skill: check.skill.clone(),
            message: finding.message,
            line: None,
            symbol: None,
            rerun: rerun.into(),
        })
        .collect();
    print_diagnostics(&diagnostics);
    Ok(i32::from(!diagnostics.is_empty()))
}
fn placement(context: &Context, tools: &[String], findings: &mut Findings) -> Result<()> {
    let settings = &context.config.tools;
    let (locations, entries) = (globs(&settings.locations)?, globs(&settings.entries)?);
    for path in tools {
        let referenced = context
            .config
            .commands
            .values()
            .any(|command| command.argv.contains(path));
        let registered = referenced || entries.is_match(path);
        let message = match (locations.is_match(path), registered) {
            (false, _) => format!(
                "tool is outside tools.locations: {}",
                settings.locations.join(", ")
            ),
            (true, false) => {
                "tool is not registered; run it from a catalog command or list it in tools.entries"
                    .into()
            }
            (true, true) => continue,
        };
        findings.push(Finding {
            path: path.clone(),
            message,
        });
    }
    for pattern in &settings.entries {
        let entry = globs(std::slice::from_ref(pattern))?;
        let stale = !tools.iter().any(|path| entry.is_match(path));
        if stale {
            findings.push(Finding {
                path: module("tools"),
                message: format!("tools.entries {pattern} matches no tool"),
            });
        }
    }
    Ok(())
}
fn catalog(context: &Context, findings: &mut Findings) {
    for (name, command) in &context.config.commands {
        let undescribed = command.purpose.trim().is_empty();
        if undescribed {
            findings.push(Finding {
                path: module("commands"),
                message: format!("commands.{name}: purpose is required"),
            });
        }
        let references = command.argv.iter().filter(|argument| path_like(argument));
        for reference in references.chain(&command.doc) {
            let target = reference.split('#').next().unwrap_or_default();
            let missing = !context.root.join(target).exists();
            if missing {
                findings.push(Finding {
                    path: module("commands"),
                    message: format!("commands.{name}: {target} does not exist"),
                });
            }
        }
    }
}
fn path_like(argument: &str) -> bool {
    argument.contains('/')
        && !argument.starts_with(['-', '/'])
        && !argument.contains("://")
        && !argument.contains(char::is_whitespace)
}
fn record_problems(context: &Context, name: &str, item: &Removed) -> Vec<String> {
    let entry = format!("tools.removed.{name}");
    let mut problems = Vec::new();
    let incomplete = item.purpose.trim().is_empty() || item.reason.trim().is_empty();
    if incomplete {
        problems.push(format!("{entry}: purpose and reason are required"));
    }
    let active = context.config.commands.contains_key(name);
    if active {
        problems.push(format!("{entry} is also an active catalog command"));
    }
    let present = context.root.join(&item.path).exists();
    if present {
        problems.push(format!("{entry}: {} still exists", item.path));
    }
    let repository = crate::shared::vcs::Repository::new(&context.root);
    let unrestorable = item
        .revision
        .as_ref()
        .is_some_and(|revision| repository.read(revision, &item.path).is_err());
    if unrestorable {
        problems.push(format!("{entry}: revision does not contain {}", item.path));
    }
    problems
}
fn removed(context: &Context, files: &[String], findings: &mut Findings) {
    let memory = Path::new(&context.config.paths.memory);
    let registry = module("tools");
    let readers: Vec<_> = files
        .iter()
        .filter(|file| !Path::new(file).starts_with(memory) && **file != registry)
        .filter_map(|file| Some((file, fs::read_to_string(context.root.join(file)).ok()?)))
        .collect();
    for (name, item) in &context.config.tools.removed {
        findings.extend(
            record_problems(context, name, item)
                .into_iter()
                .map(|message| Finding {
                    path: registry.clone(),
                    message,
                }),
        );
        for (file, text) in &readers {
            let referenced = text.contains(&item.path);
            if referenced {
                findings.push(Finding {
                    path: (*file).clone(),
                    message: format!("references removed tool {}", item.path),
                });
            }
        }
    }
}
