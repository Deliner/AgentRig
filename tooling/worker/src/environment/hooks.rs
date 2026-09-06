use anyhow::{Context, Result, ensure};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{collections::BTreeMap, path::PathBuf};

#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Hook {
    pub event: Event,
    pub program: String,
    #[serde(default)]
    pub args: Vec<String>,
    pub matcher: Option<String>,
    pub timeout_seconds: u64,
}

#[derive(Clone, Deserialize, Serialize)]
pub enum Event {
    SessionStart,
    PreToolUse,
    PostToolUse,
    Stop,
}

pub fn validate(
    hooks: &BTreeMap<String, Hook>,
    programs: &BTreeMap<String, PathBuf>,
) -> Result<()> {
    for (name, hook) in hooks {
        ensure!(
            review_runner::config::identifier(name),
            "invalid hook identifier {name}"
        );
        ensure!(
            programs.contains_key(&hook.program),
            "hook {name} references unknown program {}",
            hook.program
        );
        ensure!(
            hook.timeout_seconds > 0,
            "hook {name} timeout_seconds must be positive"
        );
        if let Some(matcher) = &hook.matcher {
            ensure!(
                !matches!(hook.event, Event::Stop),
                "hook {name}: Stop does not support matcher"
            );
            let validate_regex = matcher != "*";
            if validate_regex {
                regex::Regex::new(matcher).with_context(|| format!("hook {name} matcher"))?;
            }
        }
    }
    Ok(())
}

pub fn configuration(
    hooks: &BTreeMap<String, Hook>,
    command: impl Fn(&str, &Hook) -> String,
) -> Value {
    let mut events = serde_json::Map::new();
    for (name, hook) in hooks {
        let handler = json!({"type":"command", "command":command(name, hook), "timeout":hook.timeout_seconds});
        let mut group = json!({"hooks":[handler]});
        if let Some(matcher) = &hook.matcher {
            group["matcher"] = json!(matcher);
        }
        let event = serde_json::to_value(&hook.event)
            .unwrap()
            .as_str()
            .unwrap()
            .to_owned();
        events
            .entry(event)
            .or_insert_with(|| json!([]))
            .as_array_mut()
            .unwrap()
            .push(group);
    }
    Value::Object(events)
}
