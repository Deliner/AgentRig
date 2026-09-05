mod model;
mod plan;
mod release;
mod storage;

use anyhow::{Result, bail};
use std::path::Path;

pub fn run(root: &Path, args: &[String]) -> Result<i32> {
    match args {
        [command, binary] if command == "plan" => plan::create(root, Path::new(binary)),
        _ => bail!("upgrade plan RELEASE_EXECUTABLE"),
    }
}
