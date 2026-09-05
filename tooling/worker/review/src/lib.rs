pub mod config;
pub mod contract;
pub mod response;

pub fn digest(bytes: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    format!("{:x}", Sha256::digest(bytes))
}
pub mod execution;
pub mod run;
pub mod snapshot;

pub mod mcp;

pub mod cli;
