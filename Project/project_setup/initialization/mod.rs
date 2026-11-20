// DECISION: D033
mod no_overwrite_init;
#[cfg(test)]
mod tests;
mod wizard;
use anyhow::Result;
use std::path::Path;

pub fn init(root: &Path, args: &[String]) -> Result<i32> {
    let interactive = args == ["--interactive"];
    if interactive {
        return wizard::run(root);
    }
    no_overwrite_init::run(root, args)
}
