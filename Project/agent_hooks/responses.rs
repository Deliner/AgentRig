use serde_json::{Value, json};

#[derive(Clone, Copy)]
pub enum Event {
    SessionStart,
    PreToolUse,
}
impl Event {
    fn name(self) -> &'static str {
        match self {
            Self::SessionStart => "SessionStart",
            Self::PreToolUse => "PreToolUse",
        }
    }
}
pub fn context(event: Event, message: &str) -> Value {
    json!({"hookSpecificOutput": {"hookEventName": event.name(), "additionalContext": message}})
}
pub fn deny(reason: &str) -> Value {
    json!({"hookSpecificOutput": {"hookEventName": "PreToolUse",
        "permissionDecision": "deny", "permissionDecisionReason": reason}})
}
