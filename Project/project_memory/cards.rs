// DECISION: D012
// DECISION: D031
// DECISION: D242
use super::format::sections_of;
use crate::project_config::{Context, Key};
use crate::shared::fs_with_context as fs;
use crate::shared::vcs::Repository;
use anyhow::{Context as _, Result, bail, ensure};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::{Path, PathBuf},
};

const DIRECTORIES: [&str; 3] = ["Backlog", PLAN, ARCHIVE];
const PLAN: &str = "Plan";
const ARCHIVE: &str = "Archive";
const SECTIONS: [&str; 3] = ["Context", "Proposal", "Expected benefit"];
const MATERIALS: [&str; 3] = ["assets", "results", "scenarios.md"];
const PREREQUISITES: &str = "After: ";
const REGISTRIES: [&str; 4] = ["Plan.md", "Plan", "Backlog.md", "Backlog"];

struct Card {
    entry: PathBuf,
    shown: PathBuf,
    directory: bool,
}

pub fn check(context: &Context, memory: &Path) -> Result<()> {
    let entries = entries(memory)?;
    let none = entries.is_empty();
    if none {
        return Ok(());
    }
    let key = context.config.queue_key()?;
    let cards = numbered(&context.root, key, entries)?;
    let mut order = BTreeMap::new();
    for (number, card) in &cards {
        let after = card.prerequisites(key, *number)?;
        for prior in &after {
            ensure!(
                cards.contains_key(prior),
                "{}: After names {key}-{prior}, which is no card in {}",
                card.shown.display(),
                DIRECTORIES.join(", ")
            );
        }
        order.insert(*number, after);
    }
    acyclic(key, &order)
}
pub fn active_card(context: &Context) -> Result<Option<PathBuf>> {
    let Some(repository) = Repository::discover(&context.root)? else {
        return Ok(None);
    };
    card_of(context, &repository.observe()?.branch)
}
pub fn card_of(context: &Context, branch: &str) -> Result<Option<PathBuf>> {
    let memory = context.path(&context.config.paths.memory)?;
    let Some(card) = carried(context, branch) else {
        return Ok(None);
    };
    let places = DIRECTORIES.map(|directory| memory.join(directory));
    let forms = places
        .iter()
        .flat_map(|place| [place.join(format!("{card}.md")), place.join(&card)]);
    for path in forms {
        let held = fs::try_exists(&path)?;
        if held {
            return Ok(Some(path));
        }
    }
    Ok(None)
}
pub fn planned_cards(context: &Context) -> Result<Vec<String>> {
    let memory = context.path(&context.config.paths.memory)?;
    let key = context.config.queue_key()?;
    let mut numbers = BTreeSet::new();
    for entry in listed(&memory.join(PLAN))? {
        numbers.insert(Card::at(&context.root, entry)?.number(key)?);
    }
    Ok(numbers
        .iter()
        .map(|number| format!("{key}-{number}"))
        .collect())
}
fn carried(context: &Context, branch: &str) -> Option<String> {
    let key = context.config.key.as_ref()?;
    let name = branch.strip_prefix(context.config.git.prefix.as_str())?;
    let numbered = name.strip_prefix(&format!("{key}-"))?;
    let digits = numbered
        .split_once('-')
        .map_or(numbered, |(digits, _)| digits);
    number(digits).map(|number| format!("{key}-{number}"))
}
fn entries(memory: &Path) -> Result<Vec<PathBuf>> {
    let mut found = Vec::new();
    for directory in DIRECTORIES {
        let archive = directory == ARCHIVE;
        let registries: &[&str] = if archive { &REGISTRIES } else { &[] };
        let listed = listed(&memory.join(directory))?;
        let cards = listed
            .into_iter()
            .filter(|path| !registries.iter().any(|name| path.ends_with(name)));
        found.extend(cards);
    }
    found.sort();
    Ok(found)
}
fn listed(directory: &Path) -> Result<Vec<PathBuf>> {
    let Some(entries) = fs::present(fs::read_dir(directory))? else {
        return Ok(Vec::new());
    };
    entries.map(|entry| Ok(entry?.path())).collect()
}
fn numbered(root: &Path, key: &Key, entries: Vec<PathBuf>) -> Result<BTreeMap<u32, Card>> {
    let mut cards: BTreeMap<u32, Card> = BTreeMap::new();
    for entry in entries {
        let card = Card::at(root, entry)?;
        let number = card.number(key)?;
        if let Some(prior) = cards.get(&number) {
            bail!(
                "{key}-{number} lies in both {} and {}",
                prior.shown.display(),
                card.shown.display()
            );
        }
        cards.insert(number, card);
    }
    Ok(cards)
}
impl Card {
    fn at(root: &Path, entry: PathBuf) -> Result<Self> {
        let shown = entry.trim_prefix(root).to_path_buf();
        let directory = fs::metadata(&entry)?.is_dir();
        Ok(Self {
            entry,
            shown,
            directory,
        })
    }
    fn number(&self, key: &Key) -> Result<u32> {
        let name = self
            .entry
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or_default();
        let stem = if self.directory {
            Some(name)
        } else {
            name.strip_suffix(".md")
        };
        stem.and_then(|stem| parse(key, stem)).with_context(|| {
            let hint = suggested(name);
            format!(
                "{}: a card is named {key}-{hint}.md or {key}-{hint}/",
                self.shown.display()
            )
        })
    }
    fn prerequisites(&self, key: &Key, number: u32) -> Result<BTreeSet<u32>> {
        let text = self.text(key, number)?;
        let preamble = text.lines().take_while(|line| !line.starts_with("## "));
        let mut after = BTreeSet::new();
        for names in preamble.filter_map(|line| line.strip_prefix(PREREQUISITES)) {
            for name in names.split(',').map(str::trim) {
                let prior = parse(key, name).with_context(|| {
                    let shown = self.shown.display();
                    format!("{shown}: After names {name}, which is not a card name {key}-NNN")
                })?;
                after.insert(prior);
            }
        }
        Ok(after)
    }
    fn text(&self, key: &Key, number: u32) -> Result<String> {
        let card = format!("{key}-{number}.md");
        let shown = self.shown.display();
        let file = if self.directory {
            self.materials(&card)?;
            self.entry.join(&card)
        } else {
            self.entry.clone()
        };
        let Some(text) = fs::present(fs::read_to_string(&file))? else {
            bail!("{shown}: a card directory holds its card {card}");
        };
        let heading = format!("# {key}-{number} — ");
        let titled = text
            .lines()
            .next()
            .and_then(|line| line.strip_prefix(&heading))
            .is_some_and(|title| !title.trim().is_empty());
        ensure!(
            titled,
            "{shown}: the first line of a card is \"{heading}\" followed by its title"
        );
        sections_of(&text, &file, &SECTIONS, None)?;
        Ok(text)
    }
    fn materials(&self, card: &str) -> Result<()> {
        for entry in fs::read_dir(&self.entry)? {
            let name = entry?.file_name();
            let held = name == card || MATERIALS.iter().any(|material| name == *material);
            ensure!(
                held,
                "{}: a card directory holds only {card}, assets/, results/ and scenarios.md",
                self.shown.join(name).display()
            );
        }
        Ok(())
    }
}
fn parse(key: &Key, name: &str) -> Option<u32> {
    number(name.strip_prefix(&format!("{key}-"))?)
}
fn number(digits: &str) -> Option<u32> {
    let number: u32 = digits.parse().ok()?;
    let canonical = number.to_string() == digits;
    canonical.then_some(number)
}
fn suggested(name: &str) -> String {
    let digits: String = name.chars().filter(char::is_ascii_digit).collect();
    let number: Option<u32> = digits.parse().ok();
    number.map_or_else(|| "NNN".into(), |number| number.to_string())
}
// DECISION: D127
fn acyclic(key: &Key, order: &BTreeMap<u32, BTreeSet<u32>>) -> Result<()> {
    let mut done = BTreeSet::new();
    while done.len() < order.len() {
        let before = done.len();
        for (number, after) in order {
            let ready = after.iter().all(|prior| done.contains(prior));
            if ready {
                done.insert(*number);
            }
        }
        let progressed = done.len() > before;
        ensure!(
            progressed,
            "After forms a cycle that leaves {} waiting",
            waiting(key, order, &done)
        );
    }
    Ok(())
}
fn waiting(key: &Key, order: &BTreeMap<u32, BTreeSet<u32>>, done: &BTreeSet<u32>) -> String {
    let waiting = order.keys().filter(|number| !done.contains(*number));
    let names: Vec<String> = waiting.map(|number| format!("{key}-{number}")).collect();
    names.join(", ")
}
