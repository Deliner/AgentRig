use super as run;
use super::super::config;
use anyhow::{Context, Result};
use review_runner::mcp::protocol::{self, Session};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::path::Path;

#[derive(Clone, Copy, Deserialize, Serialize)]
enum Tool {
    #[serde(rename = "delegate_start")]
    Start,
    #[serde(rename = "delegate_status")]
    Status,
    #[serde(rename = "delegate_result")]
    Result,
    #[serde(rename = "delegate_cancel")]
    Cancel,
}
const TOOLS: [Tool; 4] = [Tool::Start, Tool::Status, Tool::Result, Tool::Cancel];

pub fn serve(root: &Path, runtime: &Path, path: &Path) -> Result<()> {
    let config = config::load(path)?;
    let mut session = Session::default();
    protocol::serve(|value| {
        session.message(value, initialize(), |method, params| match method {
            "tools/list" => Ok(list(&config)),
            _ => call((root, runtime, path), &config, params),
        })
    })
}

fn list(config: &config::Config) -> Value {
    let tools: Vec<_> = TOOLS.iter().map(|tool| json!({"name":tool,"description":tool.description(),
        "inputSchema":tool.schema(config),"annotations":{"readOnlyHint":false,"destructiveHint":false,
        "idempotentHint":!matches!(tool, Tool::Start),"openWorldHint":true}})).collect();
    json!({"tools":tools})
}

fn call(paths: (&Path, &Path, &Path), config: &config::Config, params: &Value) -> Result<Value> {
    let tool: Tool =
        serde_json::from_value(params["name"].clone()).context("unknown delegation tool")?;
    let arguments = &params["arguments"];
    jsonschema::validator_for(&tool.schema(config))?
        .validate(arguments)
        .map_err(|error| anyhow::anyhow!("delegate arguments: {error}"))?;
    Ok(match tool.execute(paths, arguments) {
        Ok(value) => {
            let failed = matches!(
                value["outcome"].as_str(),
                Some("ERROR" | "CANCELLED" | "UNKNOWN")
            );
            json!({"content":[{"type":"text","text":serde_json::to_string(&value)?}],"structuredContent":value,"isError":failed})
        }
        Err(error) => {
            json!({"content":[{"type":"text","text":format!("{error:#}")}],"isError":true})
        }
    })
}

impl Tool {
    fn description(self) -> &'static str {
        match self {
            Self::Start => {
                "Start an isolated task using a configured profile; retain run_id for polling."
            }
            Self::Status => {
                "Observe a managed delegate and recover a terminal report or incomplete cleanup."
            }
            Self::Result => {
                "Retrieve the retained delegate report and artifact manifest after reconnecting."
            }
            Self::Cancel => {
                "Cancel an owned delegate scope and recover its report and temporary-file cleanup."
            }
        }
    }
    fn schema(self, config: &config::Config) -> Value {
        let starting = matches!(self, Self::Start);
        if starting {
            let mut schema: Value =
                serde_json::from_str(include_str!("start.json")).expect("embedded delegate schema");
            schema["properties"]["profile"]["enum"] =
                json!(config.profiles.keys().collect::<Vec<_>>());
            schema
        } else {
            json!({"type":"object","required":["run_id"],"additionalProperties":false,
                "properties":{"run_id":{"type":"string","pattern":"^[A-Za-z0-9_-]+$"}}})
        }
    }
    fn execute(self, paths: (&Path, &Path, &Path), arguments: &Value) -> Result<Value> {
        let (root, runtime, config) = paths;
        match self {
            Self::Start => run::start(
                root,
                runtime,
                config,
                serde_json::from_value(arguments.clone())?,
            ),
            Self::Status | Self::Result => {
                run::result(runtime, arguments["run_id"].as_str().unwrap())
            }
            Self::Cancel => {
                let id = arguments["run_id"].as_str().unwrap();
                run::cancel(runtime, id)
            }
        }
    }
}
fn initialize() -> Value {
    json!({"protocolVersion":"2025-11-25","capabilities":{"tools":{"listChanged":false}},
        "serverInfo":{"name":"worker-delegation","version":env!("CARGO_PKG_VERSION")},
        "instructions":"Use configured profiles and explicit task contracts. Retain run_id; poll status/result after reconnecting. The top-level outcome determines success. This service does not merge or publish changes."})
}
