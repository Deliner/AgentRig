// DECISION: D047
use super::{context, deny};
use crate::project_config::{Context, RouteEvent};
use crate::shared::paths::patterns::globs;
use anyhow::Result;
use serde_json::{Value, json};
use std::path::{Path, PathBuf};

pub struct Occurrence {
    route: usize,
    area: String,
    reason: String,
}
pub fn edits(configured: &Context, paths: &[PathBuf]) -> Result<Vec<Occurrence>> {
    let mut found = Vec::new();
    for (route, settings) in configured.config.hooks.routes.iter().enumerate() {
        let include = globs(&settings.include)?;
        let listens = settings.events.contains(&RouteEvent::Edit);
        for path in paths
            .iter()
            .filter(|path| listens && include.is_match(path))
        {
            let area = path
                .parent()
                .filter(|parent| !parent.as_os_str().is_empty())
                .unwrap_or(Path::new("."));
            found.push(Occurrence {
                route,
                area: area.display().to_string(),
                reason: format!("edit of {}", path.display()),
            });
        }
    }
    Ok(found)
}
pub fn events(configured: &Context, event: RouteEvent, reason: &str) -> Vec<Occurrence> {
    let routes = configured.config.hooks.routes.iter().enumerate();
    routes
        .filter(|(_, settings)| settings.events.contains(&event))
        .map(|(route, _)| Occurrence {
            route,
            area: reason.into(),
            reason: reason.into(),
        })
        .collect()
}
pub fn pending(state: &mut Value, configured: &Context, found: Vec<Occurrence>) -> Vec<Occurrence> {
    let delivered = state["delivered"].as_array().cloned().unwrap_or_default();
    let mut keys = delivered.clone();
    let mut fresh = Vec::new();
    for occurrence in found {
        let skill = &configured.config.hooks.routes[occurrence.route].skill;
        let key = json!([skill, occurrence.area]);
        let undelivered = !delivered.contains(&key);
        if undelivered {
            let unrecorded = !keys.contains(&key);
            if unrecorded {
                keys.push(key);
            }
            fresh.push(occurrence);
        }
    }
    state["delivered"] = Value::Array(keys);
    fresh
}

pub struct Answer {
    pub discipline: Option<String>,
    pub found: Vec<Occurrence>,
    pub notes: Vec<String>,
    pub tracked: bool,
}
impl Answer {
    pub fn output(mut self, configured: &Context, tool: &str) -> Result<Option<Value>> {
        let routes = &configured.config.hooks.routes;
        self.found
            .sort_by(|left, right| (left.route, &left.reason).cmp(&(right.route, &right.reason)));
        let mut blocking = self.discipline.is_some();
        let mut lines: Vec<String> = self.discipline.into_iter().collect();
        let same_skill = |left: &Occurrence, right: &Occurrence| {
            routes[left.route].skill == routes[right.route].skill
        };
        for group in self.found.chunk_by(same_skill) {
            let required = self.tracked && group.iter().any(|item| routes[item.route].block);
            blocking |= required;
            let enforcement = if required {
                Enforcement::Required
            } else {
                Enforcement::Guidance
            };
            lines.push(requirement(configured, group, enforcement)?);
        }
        lines.extend(self.notes);
        let silent = lines.is_empty();
        if silent {
            return Ok(None);
        }
        let status = if blocking {
            format!(
                "AgentRig stopped this {tool} call before it ran. Apply the requirements below in order, then retry the same call; delivered requirements do not stop it again."
            )
        } else {
            format!("AgentRig allowed this {tool} call. Apply the guidance below to the change.")
        };
        let message = format!("{status}\n\n{}", lines.join("\n\n"));
        Ok(Some(if blocking {
            deny(&message)
        } else {
            context("PreToolUse", &message)
        }))
    }
}
// DECISION: D062
#[derive(Clone, Copy)]
enum Enforcement {
    Required,
    Guidance,
}
impl Enforcement {
    fn label(self) -> &'static str {
        match self {
            Self::Required => "Required",
            Self::Guidance => "Guidance",
        }
    }
}
fn requirement(
    configured: &Context,
    group: &[Occurrence],
    enforcement: Enforcement,
) -> Result<String> {
    let routes = &configured.config.hooks.routes;
    let skill = configured.path(&routes[group[0].route].skill)?;
    let mut reasons: Vec<&str> = group.iter().map(|item| item.reason.as_str()).collect();
    reasons.dedup();
    let mut sources: Vec<String> = group
        .iter()
        .map(|item| format!("hooks.routes[{}]", item.route))
        .collect();
    sources.dedup();
    let kind = enforcement.label();
    Ok(format!(
        "- {kind}: read and apply {} for {}. Source: {}.",
        skill.display(),
        reasons.join("; "),
        sources.join(", ")
    ))
}
