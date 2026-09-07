use sha2::{Digest, Sha256};
pub mod config;
pub use response::contract;
pub mod response;

pub fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
pub mod execution;
pub mod run;
pub mod snapshot;
pub mod vcs;

pub mod mcp;

pub mod cli;
