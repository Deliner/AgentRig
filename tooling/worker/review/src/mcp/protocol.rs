use anyhow::Result;
use serde_json::{Value, json};
use std::io::{BufRead, Write};

#[derive(Default)]
pub struct Session {
    initialized: bool,
}
impl Session {
    pub fn message(
        &mut self,
        value: Value,
        initialize: Value,
        tools: impl Fn(&str, &Value) -> Result<Value>,
    ) -> Option<Value> {
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
                Ok(initialize)
            }
            "ping" => Ok(json!({})),
            _ => self.dispatch(method, &value["params"], tools),
        };
        Some(match result {
            Ok(result) => json!({"jsonrpc":"2.0","id":id,"result":result}),
            Err(failure) => error(id, failure.0, &failure.1),
        })
    }
    fn dispatch(
        &self,
        method: &str,
        params: &Value,
        tools: impl Fn(&str, &Value) -> Result<Value>,
    ) -> std::result::Result<Value, (i32, String)> {
        let uninitialized = !self.initialized;
        if uninitialized {
            return Err((-32000, "Initialize the session first".into()));
        }
        match method {
            "tools/list" | "tools/call" => {
                tools(method, params).map_err(|error| (-32602, format!("{error:#}")))
            }
            _ => Err((-32601, "Method not found".into())),
        }
    }
}
fn error(id: Value, code: i32, message: &str) -> Value {
    json!({"jsonrpc":"2.0","id":id,"error":{"code":code,"message":message}})
}
pub fn serve(mut handler: impl FnMut(Value) -> Option<Value>) -> Result<()> {
    let stdin = std::io::stdin();
    let mut stdout = std::io::stdout().lock();
    for line in stdin.lock().lines() {
        let response = match serde_json::from_str(&line?) {
            Ok(value) => handler(value),
            Err(_) => Some(error(Value::Null, -32700, "Parse error")),
        };
        if let Some(response) = response {
            writeln!(stdout, "{}", serde_json::to_string(&response)?)?;
            stdout.flush()?;
        }
    }
    Ok(())
}
