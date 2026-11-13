use super::Texts;
use super::clusters;
use super::words::{split, stem};
use crate::shared::source_parsers;
use crate::structural_lint::languages::{Code, in_test_context, text};
use anyhow::Result;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use tree_sitter::Node;

pub type Counts = BTreeMap<String, usize>;

pub(super) const IDENTIFIERS: &[&str] = &[
    "identifier",
    "type_identifier",
    "field_identifier",
    "shorthand_field_identifier",
];
const QUALIFIED: &[&str] = &["scoped_identifier", "scoped_type_identifier"];

pub struct Source {
    pub words: Counts,
    pub about: Counts,
    pub clusters: usize,
}

#[derive(Default)]
pub struct Corpus {
    pub files: BTreeMap<PathBuf, Source>,
    pub descriptions: BTreeMap<PathBuf, Counts>,
    pub surfaces: BTreeMap<String, Counts>,
}

impl Corpus {
    pub fn new(texts: &Texts) -> Result<Self> {
        let mut corpus = Self::default();
        for (path, text) in &texts.code {
            let source = corpus.source(path, text)?;
            corpus.files.insert(path.clone(), source);
        }
        for (directory, contract) in &texts.contracts {
            let purpose = purpose(contract).map(|text| (directory.clone(), text));
            let files = file_entries(contract).into_iter();
            let described = files.map(|(name, text)| (directory.join(name), text));
            for (path, text) in purpose.into_iter().chain(described) {
                let mut words = Counts::new();
                corpus.record(text, &mut words);
                corpus.descriptions.insert(path, words);
            }
        }
        Ok(corpus)
    }

    fn source(&mut self, path: &Path, text: &str) -> Result<Source> {
        let mut source = Source {
            words: Counts::new(),
            about: Counts::new(),
            clusters: 0,
        };
        let Some(tree) = source_parsers::parse(path, text)? else {
            return Ok(source);
        };
        self.record(&summary(text), &mut source.about);
        let code = Code::new(text);
        let mut pending = vec![tree.root_node()];
        while let Some(node) = pending.pop() {
            let elsewhere = names_other_code(node);
            if elsewhere {
                continue;
            }
            if let Some(raw) = term(node, code) {
                self.record(raw, &mut source.words);
            }
            let mut cursor = node.walk();
            pending.extend(node.children(&mut cursor));
        }
        source.clusters = clusters::count(tree.root_node(), code);
        Ok(source)
    }

    fn record(&mut self, raw: &str, words: &mut Counts) {
        for surface in split(raw) {
            if let Some(stem) = stem(&surface) {
                *words.entry(stem.clone()).or_default() += 1;
                let forms = self.surfaces.entry(stem).or_default();
                *forms.entry(surface).or_default() += 1;
            }
        }
    }
}

fn purpose(contract: &str) -> Option<&str> {
    contract
        .lines()
        .find_map(|line| line.strip_prefix("purpose:"))
        .map(str::trim)
}

fn file_entries(contract: &str) -> Vec<(&str, &str)> {
    let lines = contract
        .lines()
        .skip_while(|line| line.trim_end() != "files:")
        .skip(1);
    lines
        .take_while(|line| line.starts_with(' '))
        .filter_map(|line| line.trim().split_once(": "))
        .collect()
}

fn names_other_code(node: Node<'_>) -> bool {
    match node.kind() {
        "use_declaration" | "extern_crate_declaration" => true,
        "mod_item" => node.child_by_field_name("body").is_none(),
        _ => false,
    }
}

fn summary(text: &str) -> String {
    let opening = |line: &&str| line.starts_with("//!") || line.starts_with("/*!");
    let mut lines = text
        .lines()
        .map(str::trim_start)
        .skip_while(|line| !opening(line));
    let Some(first) = lines.next() else {
        return String::new();
    };
    let mut docs = Vec::new();
    if let Some(block) = first.strip_prefix("/*!") {
        for line in std::iter::once(block).chain(lines) {
            let closed = line.split_once("*/");
            docs.push(closed.map_or(line, |(inside, _)| inside));
            let ended = closed.is_some();
            if ended {
                break;
            }
        }
    } else {
        docs.extend(
            std::iter::once(first)
                .chain(lines)
                .map_while(|line| line.strip_prefix("//!")),
        );
    }
    let paragraph = docs
        .into_iter()
        .skip_while(|line| line.trim().is_empty())
        .take_while(|line| !line.trim().is_empty());
    paragraph.collect::<Vec<_>>().join(" ")
}

fn term<'a>(node: Node<'_>, code: Code<'a>) -> Option<&'a str> {
    let kind = node.kind();
    let raw = text(node, code);
    let identifier =
        IDENTIFIERS.contains(&kind) && !qualifier(node) && !in_test_context(node, code);
    let literal = kind == "string_content" && !in_test_context(node, code);
    let comment = matches!(kind, "line_comment" | "block_comment") && documentation(raw);
    (identifier || literal || comment).then_some(raw)
}

fn qualifier(node: Node<'_>) -> bool {
    node.parent().is_some_and(|parent| {
        QUALIFIED.contains(&parent.kind()) && parent.child_by_field_name("path") == Some(node)
    })
}

fn documentation(comment: &str) -> bool {
    let outer_line = comment.starts_with("///") && !comment.starts_with("////");
    let outer_block =
        comment.starts_with("/**") && !comment.starts_with("/***") && comment != "/**/";
    let inner = comment.starts_with("//!") || comment.starts_with("/*!");
    let outer = outer_line || outer_block;
    outer || inner
}
