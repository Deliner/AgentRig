use crate::{config, run};
pub mod protocol;
use anyhow::{Context, Result, ensure};
use serde_json::{Value, json};
use std::path::{Path, PathBuf};

pub const ARGUMENTS: &str = include_str!("arguments.json");
pub struct Server {
    config: config::Config,
    path: PathBuf,
    session: protocol::Session,
}
impl Server {
    pub fn new(path: &Path) -> Result<Self> {
        let path = path.canonicalize()?;
        Ok(Self {
            config: config::load(&path)?,
            path,
            session: protocol::Session::default(),
        })
    }
    pub fn message(&mut self, value: Value) -> Option<Value> {
        let config = &self.config;
        let path = &self.path;
        self.session
            .message(value, initialize(), |method, params| match method {
                "tools/list" => Ok(Self::list(config)),
                _ => Self::call(config, path, params),
            })
    }
    fn list(config: &config::Config) -> Value {
        let schema: Value = serde_json::from_str(ARGUMENTS).expect("embedded argument schema");
        let tools: Vec<_> = config.tools.iter().map(|(name, tool)| json!({
            "name":name,"description":tool.description,"inputSchema":schema,
            "annotations":{"readOnlyHint":false,"destructiveHint":false,"idempotentHint":false,"openWorldHint":true}
        })).collect();
        json!({"tools":tools})
    }
    fn call(config: &config::Config, path: &Path, params: &Value) -> Result<Value> {
        let name = params["name"].as_str().context("tool name required")?;
        ensure!(
            config.tools.contains_key(name),
            "unknown review tool: {name}"
        );
        let request = arguments(name, params["arguments"].clone())?;
        Ok(match run::run(path, request) {
            Ok(report) => {
                json!({"content":[{"type":"text","text":serde_json::to_string(&report)?}],
                "structuredContent":report,"isError":report.verdict == "BLOCKED" || report.cleanup_error.is_some()})
            }
            Err(error) => {
                json!({"content":[{"type":"text","text":format!("{error:#}")}],"isError":true})
            }
        })
    }
}
pub fn arguments(name: &str, mut value: Value) -> Result<run::Request> {
    let schema: Value = serde_json::from_str(ARGUMENTS)?;
    jsonschema::validator_for(&schema)?
        .validate(&value)
        .map_err(|error| anyhow::anyhow!("{error}"))?;
    value
        .as_object_mut()
        .context("arguments must be an object")?
        .insert("tool".into(), name.into());
    Ok(serde_json::from_value(value)?)
}
fn initialize() -> Value {
    json!({"protocolVersion":"2025-11-25","capabilities":{"tools":{"listChanged":false}},
        "serverInfo":{"name":"review-mcp","version":env!("CARGO_PKG_VERSION")},
        "instructions":"Call a configured review with an exact Git boundary. Configure the client tool timeout above runner.timeout_seconds plus preparation/report overhead. Reports preserve findings; the service does not modify the project."})
}
pub fn serve(path: &Path) -> Result<()> {
    let mut server = Server::new(path)?;
    protocol::serve(|value| server.message(value))
}
