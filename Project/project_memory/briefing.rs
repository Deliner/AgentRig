use crate::shared::fs_with_context as fs;
use std::path::Path;

pub const NORTH_STAR: &str = "NorthStar";
pub const GLOSSARY: &str = "Glossary";

pub fn briefing(memory: &Path) -> String {
    [NORTH_STAR, GLOSSARY]
        .map(|name| given(&memory.join(format!("{name}.md"))))
        .join("\n\n")
}

fn given(path: &Path) -> String {
    let shown = path.display();
    match fs::present(fs::read_to_string(path)) {
        Ok(Some(text)) => {
            let text = text.trim_end();
            format!("{shown}:\n\n{text}")
        }
        Ok(None) => format!("{shown} is missing, so this session starts without it."),
        Err(error) => {
            format!("{shown} is unreadable, so this session starts without it: {error:#}")
        }
    }
}
