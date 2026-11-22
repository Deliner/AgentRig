// DECISION: D033
mod branches;
mod directories;
mod export;
mod git;
mod hooks;
mod objects;
mod observation;
mod repository;
#[cfg(test)]
mod tests;
mod tree_entry;
pub use branches::Settings;
pub use git::{Revisions, Staging, context_text as git_context};
pub use hooks::{Generation, generate};
pub use observation::Observation;
pub use repository::{Repository, initialize};
pub use tree_entry::{Entry, FileKind};
