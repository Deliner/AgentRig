// DECISION: D225
use super::measured::measured;
use crate::structural_lint::tests::architecture::{Consumer, Language};

const RULE: &str = "file-cycle";
const LANGUAGES: [Language; 2] = [Language::Rust, Language::Python];

fn extension(language: Language) -> &'static str {
    match language {
        Language::Rust => "rs",
        Language::Python => "py",
    }
}

fn uses(language: Language, chain: &[(&str, &str)]) -> Consumer {
    let mut files = Vec::new();
    let declarations: String = chain
        .iter()
        .map(|(file, _)| format!("mod {file};\n"))
        .collect();
    if let Language::Rust = language {
        files.push(("src/lib.rs".to_owned(), "pub mod a;\n".to_owned()));
        files.push(("src/a/mod.rs".to_owned(), declarations));
    }
    for (file, used) in chain {
        let text = match language {
            Language::Rust => format!("pub fn {file}() -> i32 {{ super::{used}::{used}() }}\n"),
            Language::Python => {
                format!("from a.{used} import {used}\n\n\ndef {file}():\n    return {used}()\n")
            }
        };
        files.push((format!("src/a/{file}.{}", extension(language)), text));
    }
    measured(language, RULE, &files)
}

fn found(consumer: &Consumer) -> Vec<String> {
    let found = consumer.lint().into_iter();
    found
        .map(|item| format!("{} {}", item.rule, item.message))
        .collect()
}

#[test]
fn two_files_that_use_each_other_are_a_cycle_named_link_by_link() {
    for language in LANGUAGES {
        let consumer = uses(language, &[("b", "c"), ("c", "b")]);
        let [b, c] = ["b", "c"].map(|file| format!("src/a/{file}.{}", extension(language)));
        let cycle = format!("file-cycle file dependency cycle: {b} -> {c} -> {b}");
        let links = format!("{b}:1 uses {c}; {c}:1 uses {b}");
        assert_eq!(found(&consumer), [format!("{cycle}; {links}")]);
        consumer.configure("include: [src, 'src/**']", "include: [src]");
        assert_eq!(found(&consumer), Vec::<String>::new());
    }
}

#[test]
fn a_chain_through_four_files_is_one_finding() {
    for language in LANGUAGES {
        let chain = [("b", "c"), ("c", "d"), ("d", "e"), ("e", "b")];
        let found = found(&uses(language, &chain));
        let files =
            ["b", "c", "d", "e", "b"].map(|file| format!("src/a/{file}.{}", extension(language)));
        let cycle = format!("file-cycle file dependency cycle: {};", files.join(" -> "));
        assert_eq!(found.len(), 1, "{found:?}");
        assert!(found[0].starts_with(&cycle), "{found:?}");
    }
}

#[test]
fn the_directory_entry_and_module_declarations_take_no_part_in_a_cycle() {
    let rust = [
        ("src/lib.rs", "pub mod a;\n"),
        (
            "src/a/mod.rs",
            "mod b;\npub use b::b;\npub fn shared() -> i32 { 7 }\n",
        ),
        ("src/a/b.rs", "pub fn b() -> i32 { super::shared() }\n"),
    ];
    let python = [
        (
            "src/a/__init__.py",
            "from a.b import b\n\n\ndef shared():\n    return 7\n",
        ),
        (
            "src/a/b.py",
            "from a import shared\n\n\ndef b():\n    return shared()\n",
        ),
    ];
    for (language, files) in [(Language::Rust, &rust[..]), (Language::Python, &python[..])] {
        assert_eq!(
            found(&measured(language, RULE, files)),
            Vec::<String>::new()
        );
    }
}

#[test]
fn file_cycle_needs_an_enabled_directory_architecture_rule() {
    let consumer = uses(Language::Rust, &[("b", "c"), ("c", "b")]);
    consumer.configure(
        "kind: directory-architecture\n",
        "kind: directory-architecture\n    enabled: false\n",
    );
    consumer.invalid();
}
