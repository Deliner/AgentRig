// DECISION: D107
#![deny(unsafe_code)]
// DECISION: D042
pub mod agent_hooks;
pub(crate) mod change_delivery;
pub mod cli_dispatch;
pub mod command_runner;
pub mod project_config;
pub(crate) mod project_memory;
pub(crate) mod project_setup;
pub mod shared;
pub mod structural_lint;
