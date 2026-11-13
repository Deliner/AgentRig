use super::{Finding, Findings, removal_records};
use crate::project_config::{Check, Context, module};
use crate::shared::diagnostics::actionable_findings::{Diagnostic, Level, print_diagnostics};
use crate::shared::paths::glob_patterns::globs;
use crate::shared::vcs::Repository;
use anyhow::Result;
use std::path::Path;

pub struct Source<'a> {
    pub repository: &'a Path,
    pub exported_revision: Option<&'a str>,
}
pub fn run(context: &Context, source: Source<'_>, check: &Check, rerun: &str) -> Result<i32> {
    let repository = Repository::new(source.repository);
    let files = match source.exported_revision {
        Some(revision) => repository.tree(revision)?.into_keys().collect(),
        None => repository.working_files()?,
    };
    let tools = super::tools(context, &files)?;
    let mut findings = Findings::new();
    placement(context, &tools, &mut findings)?;
    catalog(context, &mut findings);
    removal_records::removed(context, &repository, &files, &mut findings);
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
    let other_form = argument.starts_with(['-', '/'])
        || argument.contains("://")
        || argument.contains(char::is_whitespace);
    argument.contains('/') && !other_form
}
