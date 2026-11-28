use super::format::{Catalog, sections};
use crate::project_config::{Context, Key};
use crate::shared::fs_with_context as fs;
use anyhow::{Context as _, Result, bail, ensure};
use std::{
    collections::{BTreeMap, BTreeSet, btree_map::Entry},
    ffi::OsStr,
    path::{Path, PathBuf},
};

const PLACES: [&str; 3] = ["Backlog", "Plan", "Archive"];
const MATERIALS: [&str; 3] = ["assets", "results", "scenarios.md"];

struct Card {
    number: u32,
    entry: PathBuf,
    after: Vec<u32>,
}
struct CardFile {
    number: u32,
    path: PathBuf,
}

pub fn check(context: &Context, memory: &Path) -> Result<()> {
    let Some(key) = &context.config.key else {
        return Ok(());
    };
    let mut cards = BTreeMap::new();
    for place in PLACES {
        for card in placed(key, &memory.join(place))? {
            admit(&mut cards, card)?;
        }
    }
    order(key, &cards)
}
pub(super) fn acyclic<Node: Copy + Ord>(graph: &BTreeMap<Node, Vec<Node>>) -> Result<()> {
    let mut done = BTreeSet::new();
    while done.len() < graph.len() {
        let before = done.len();
        for (id, dependencies) in graph {
            let ready = dependencies.iter().all(|id| done.contains(id));
            if ready {
                done.insert(*id);
            }
        }
        ensure!(done.len() > before, "Plan dependency cycle");
    }
    Ok(())
}
fn placed(key: &Key, place: &Path) -> Result<Vec<Card>> {
    let Some(entries) = fs::present(fs::read_dir(place))? else {
        return Ok(Vec::new());
    };
    let prefix = format!("{key}-");
    let mut cards = Vec::new();
    for entry in entries {
        let entry = entry?.path();
        let name = entry.file_name().and_then(OsStr::to_str);
        let named = name.filter(|name| name.starts_with(&prefix));
        if let Some(name) = named {
            cards.push(card(key, &entry, name)?);
        }
    }
    Ok(cards)
}
fn admit(cards: &mut BTreeMap<u32, Card>, card: Card) -> Result<()> {
    match cards.entry(card.number) {
        Entry::Vacant(slot) => {
            slot.insert(card);
            Ok(())
        }
        Entry::Occupied(slot) => {
            let first = slot.get().entry.display();
            let second = card.entry.display();
            bail!("{first} and {second} carry one card number, but a card lies in one place")
        }
    }
}
fn card(key: &Key, entry: &Path, name: &str) -> Result<Card> {
    let file = match name.strip_suffix(".md") {
        Some(stem) => CardFile {
            number: numbered(key, entry, stem)?,
            path: entry.to_path_buf(),
        },
        None => folder(key, entry, name)?,
    };
    let after = after(key, &file)?;
    Ok(Card {
        number: file.number,
        entry: entry.to_path_buf(),
        after,
    })
}
fn numbered(key: &Key, entry: &Path, stem: &str) -> Result<u32> {
    number(key, stem).with_context(|| {
        let entry = entry.display();
        format!(
            "{entry}: a card is a file {key}-N.md or a directory {key}-N, where the number N has no leading zero"
        )
    })
}
fn number(key: &Key, name: &str) -> Option<u32> {
    let digits = name.strip_prefix(&format!("{key}-"))?;
    let number: u32 = digits.parse().ok()?;
    let canonical = number.to_string() == digits;
    canonical.then_some(number)
}
fn folder(key: &Key, entry: &Path, name: &str) -> Result<CardFile> {
    let number = numbered(key, entry, name)?;
    let card = format!("{name}.md");
    let mut held = Vec::new();
    for item in fs::read_dir(entry)? {
        held.push(item?.file_name().to_string_lossy().into_owned());
    }
    let foreign = held
        .iter()
        .find(|item| **item != card && !MATERIALS.contains(&item.as_str()));
    if let Some(foreign) = foreign {
        let foreign = entry.join(foreign);
        let foreign = foreign.display();
        let materials = MATERIALS.join(", ");
        bail!("{foreign}: a card directory holds only {card}, {materials}");
    }
    let directory = entry.display();
    ensure!(
        held.contains(&card),
        "{directory}: a card directory holds its card {card}"
    );
    Ok(CardFile {
        number,
        path: entry.join(card),
    })
}
fn after(key: &Key, file: &CardFile) -> Result<Vec<u32>> {
    let layout = Catalog::Backlog.detail_sections();
    let source = sections(&file.path, layout.required, layout.optional)?;
    let path = file.path.display();
    let number = file.number;
    let title = format!("# {key}-{number} — ");
    let mut lines = source.lines();
    let titled = lines
        .next()
        .and_then(|line| line.strip_prefix(&title))
        .is_some_and(|name| !name.trim().is_empty());
    ensure!(titled, "{path}: the first line is {title}and the card name");
    let preamble: Vec<&str> = lines
        .take_while(|line| !line.starts_with("## "))
        .filter(|line| !line.trim().is_empty())
        .collect();
    let listed = match preamble.as_slice() {
        [] => Some(Vec::new()),
        [line] => listed(key, line),
        _ => None,
    };
    listed.with_context(|| {
        format!(
            "{path}: between the title and the first section stands at most one line, After: {key}-N, naming the cards it follows separated by commas"
        )
    })
}
fn listed(key: &Key, line: &str) -> Option<Vec<u32>> {
    let names = line.strip_prefix("After: ")?;
    names
        .split(',')
        .map(|name| number(key, name.trim()))
        .collect()
}
fn order(key: &Key, cards: &BTreeMap<u32, Card>) -> Result<()> {
    let mut graph = BTreeMap::new();
    for card in cards.values() {
        let missing = card
            .after
            .iter()
            .find(|number| !cards.contains_key(*number));
        if let Some(missing) = missing {
            let entry = card.entry.display();
            let places = PLACES.join(", ");
            bail!("{entry}: After names {key}-{missing}, a card in none of {places}");
        }
        graph.insert(card.number, card.after.clone());
    }
    acyclic(&graph)
}
