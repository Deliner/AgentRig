use super::{Finding, Findings};
use crate::project_config::{Context, Removed, module};
use crate::shared::fs_with_context as fs;
use crate::shared::vcs::Repository;
use std::path::Path;

fn record_problems(
    context: &Context,
    repository: &Repository<'_>,
    name: &str,
    item: &Removed,
) -> Vec<String> {
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
    let unrestorable = item
        .revision
        .as_ref()
        .is_some_and(|revision| repository.read(revision, Path::new(&item.path)).is_err());
    if unrestorable {
        problems.push(format!("{entry}: revision does not contain {}", item.path));
    }
    problems
}
pub(super) fn removed(
    context: &Context,
    repository: &Repository<'_>,
    files: &[String],
    findings: &mut Findings,
) {
    let memory = Path::new(&context.config.paths.memory);
    let registry = module("tools");
    let readers: Vec<_> = files
        .iter()
        .filter(|file| !Path::new(file).starts_with(memory) && **file != registry)
        .filter_map(|file| Some((file, fs::read_to_string(context.root.join(file)).ok()?)))
        .collect();
    for (name, item) in &context.config.tools.removed {
        findings.extend(
            record_problems(context, repository, name, item)
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
