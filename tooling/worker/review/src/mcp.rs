use crate::{config, run};
use anyhow::{Context, Result, ensure};
use serde_json::{Value, json};
use std::{
    io::{BufRead, Write},
    path::{Path, PathBuf},
};

pub const ARGUMENTS: &str = include_str!("../schemas/arguments.json");
pub struct Server {
    config: config::Config,
    path: PathBuf,
    initialized: bool,
}
impl Server {
    pub fn new(path: &Path) -> Result<Self> {
        let path = path.canonicalize()?;
        Ok(Self {
            config: config::load(&path)?,
            path,
            initialized: false,
        })
    }
    pub fn message(&mut self, value: Value) -> Option<Value> {
        let valid = value.is_object() && value["jsonrpc"] == "2.0" && value["method"].is_string();
        let invalid = !valid;
        let id = value.get("id").cloned().unwrap_or(Value::Null);
        if invalid {
            return Some(error(id, -32600, "Invalid request"));
        }
        let notification = value.get("id").is_none();
        if notification {
            return None;
        }
        let method = value["method"].as_str().unwrap_or("");
        let result = match method {
            "initialize" => {
                self.initialized = true;
                Ok(initialize())
            }
            "ping" => Ok(json!({})),
            _ => self.dispatch(method, &value["params"]),
        };
        Some(match result {
            Ok(result) => json!({"jsonrpc":"2.0","id":id,"result":result}),
            Err(failure) => error(id, failure.0, &failure.1),
        })
    }
    fn dispatch(&self, method: &str, params: &Value) -> std::result::Result<Value, (i32, String)> {
        let uninitialized = !self.initialized;
        if uninitialized {
            return Err((-32000, "Initialize the session first".into()));
        }
        match method {
            "tools/list" => Ok(self.list()),
            "tools/call" => self
                .call(params)
                .map_err(|error| (-32602, format!("{error:#}"))),
            _ => Err((-32601, "Method not found".into())),
        }
    }
    fn list(&self) -> Value {
        let schema: Value = serde_json::from_str(ARGUMENTS).expect("embedded argument schema");
        let tools: Vec<_> = self.config.tools.iter().map(|(name, tool)| json!({
            "name":name,"description":tool.description,"inputSchema":schema,
            "annotations":{"readOnlyHint":false,"destructiveHint":false,"idempotentHint":false,"openWorldHint":true}
        })).collect();
        json!({"tools":tools})
    }
    fn call(&self, params: &Value) -> Result<Value> {
        let name = params["name"].as_str().context("tool name required")?;
        ensure!(
            self.config.tools.contains_key(name),
            "unknown review tool: {name}"
        );
        let request = arguments(name, params["arguments"].clone())?;
        Ok(match run::run(&self.path, request) {
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
fn error(id: Value, code: i32, message: &str) -> Value {
    json!({"jsonrpc":"2.0","id":id,"error":{"code":code,"message":message}})
}
pub fn serve(path: &Path) -> Result<()> {
    let mut server = Server::new(path)?;
    let stdin = std::io::stdin();
    let mut stdout = std::io::stdout().lock();
    for line in stdin.lock().lines() {
        let response = match serde_json::from_str(&line?) {
            Ok(value) => server.message(value),
            Err(_) => Some(error(Value::Null, -32700, "Parse error")),
        };
        if let Some(response) = response {
            writeln!(stdout, "{}", serde_json::to_string(&response)?)?;
            stdout.flush()?;
        }
    }
    Ok(())
}
