// DECISION: D200
use super::input::text;
use serde_json::Value;

const PASSIVE: [&str; 43] = [
    "Agent",
    "AskUserQuestion",
    "CronList",
    "EndConversation",
    "EnterPlanMode",
    "ExitPlanMode",
    "Glob",
    "Grep",
    "LSP",
    "ListAgents",
    "ListMcpResourcesTool",
    "PushNotification",
    "Read",
    "ReadMcpResourceTool",
    "RemoteTrigger",
    "ReportFindings",
    "ScheduleWakeup",
    "SendMessage",
    "SendUserFile",
    "SubagentHandback",
    "TaskCreate",
    "TaskGet",
    "TaskList",
    "TaskStop",
    "TaskUpdate",
    "TodoWrite",
    "ToolSearch",
    "WaitForMcpServers",
    "WebFetch",
    "WebSearch",
    "close_agent",
    "grep_files",
    "list_dir",
    "list_mcp_resource_templates",
    "list_mcp_resources",
    "read_file",
    "read_mcp_resource",
    "request_user_input",
    "resume_agent",
    "send_input",
    "spawn_agent",
    "update_plan",
    "view_image",
];

pub(super) enum Tool {
    Shell,
    Edit,
    Passive,
    Refused,
}

pub(super) fn classify(event: &Value) -> Tool {
    let input = &event["tool_input"];
    match text(event, "tool_name") {
        "Bash" | "Shell" | "exec_command" => Tool::Shell,
        "Monitor" if input.get("command").is_some() => Tool::Shell,
        "Monitor" if input.get("ws").is_some() => Tool::Passive,
        "apply_patch" | "Edit" | "Write" | "NotebookEdit" => Tool::Edit,
        "Agent" if input["isolation"] == "worktree" => Tool::Refused,
        name if PASSIVE.contains(&name) => Tool::Passive,
        _ => Tool::Refused,
    }
}

pub(super) fn refusal(event: &Value) -> String {
    format!(
        "AgentRig refuses this {} call: its hook checks shell commands and file edits and lets through only tools known to neither run commands nor write files.",
        text(event, "tool_name")
    )
}
