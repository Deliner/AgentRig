use super::briefing::GLOSSARY;
use crate::shared::fs_with_context as fs;
use anyhow::{Context as _, Result, ensure};
use std::collections::HashMap;
use std::path::Path;

const ENTRY: &str = "`- **word** — meaning`";

pub(super) fn check(memory: &Path) -> Result<()> {
    let path = memory.join(format!("{GLOSSARY}.md"));
    let Some(text) = fs::present(fs::read_to_string(&path))? else {
        return Ok(());
    };
    let shown = path.display();
    let mut lines = (1_usize..).zip(text.lines());
    let headed = lines.next().is_some_and(|(_, line)| header(line));
    ensure!(
        headed,
        "{shown}:1: a glossary starts with a `# ` header line"
    );
    let mut defined = HashMap::new();
    for (number, line) in lines.filter(|(_, line)| !line.trim().is_empty()) {
        let word = entry(line).with_context(|| {
            format!("{shown}:{number}: a glossary entry is {ENTRY} with a word and a meaning")
        })?;
        let earlier = *defined.entry(word.to_lowercase()).or_insert(number);
        ensure!(
            earlier == number,
            "{shown}:{earlier} and {shown}:{number} define the word {word} twice; keep one entry, since words compare without letter case"
        );
    }
    Ok(())
}

fn header(line: &str) -> bool {
    line.strip_prefix("# ")
        .is_some_and(|title| !title.trim().is_empty())
}

fn entry(line: &str) -> Option<&str> {
    let (word, meaning) = line.strip_prefix("- **")?.split_once("** — ")?;
    let filled = !word.trim().is_empty() && !meaning.trim().is_empty();
    filled.then_some(word)
}
