use anyhow::{Result, bail};
#[cfg(test)]
mod tests;

pub fn take_option(args: &mut Vec<String>, name: &str) -> Result<Option<String>> {
    let Some(index) = args
        .iter()
        .take_while(|arg| arg.as_str() != "--")
        .position(|arg| arg == name)
    else {
        return Ok(None);
    };
    args.remove(index);
    let has_value = index < args.len() && !args[index].starts_with("--");
    if has_value {
        Ok(Some(args.remove(index)))
    } else {
        bail!("{name} requires a value")
    }
}
