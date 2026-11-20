// DECISION: D227
use super::measured::measured;
use crate::structural_lint::tests::architecture::{Consumer, Language};

const RULE: &str = "entry-use";

const RUST_JOBS: [(&str, &str); 9] = [
    ("src/lib.rs", "pub mod jobs;\npub mod other;\n"),
    (
        "src/jobs/mod.rs",
        "mod cleanup;\nmod guard;\nmod job;\nmod queue;\n#[cfg(test)]\nmod tests;\npub use job::Job;\npub fn limit() -> usize { 1 }\n",
    ),
    ("src/jobs/job.rs", "pub struct Job;\n"),
    (
        "src/jobs/cleanup.rs",
        "use super::{Job, limit};\npub fn clean(_: Job) -> usize { limit() }\n",
    ),
    (
        "src/jobs/queue.rs",
        "use super::job::Job;\npub fn queue(_: Job) {}\n",
    ),
    ("src/jobs/guard/mod.rs", "mod x;\n"),
    (
        "src/jobs/guard/x.rs",
        "pub fn guard(_: crate::jobs::Job) {}\n",
    ),
    ("src/jobs/tests/mod.rs", "use super::*;\n"),
    ("src/other/mod.rs", "pub fn other(_: crate::jobs::Job) {}\n"),
];

const PYTHON_JOBS: [(&str, &str); 8] = [
    (
        "src/jobs/__init__.py",
        "from jobs.job import Job\n\n\ndef limit():\n    return 1\n",
    ),
    ("src/jobs/job.py", "class Job:\n    pass\n"),
    ("src/jobs/cleanup.py", "from jobs import Job, limit\n"),
    ("src/jobs/queue.py", "from jobs.job import Job\n"),
    ("src/jobs/guard/__init__.py", ""),
    ("src/jobs/guard/x.py", "from jobs import Job\n"),
    ("src/jobs/tests/test_jobs.py", "from jobs import Job\n"),
    ("src/other/__init__.py", "from jobs import Job\n"),
];

fn found(consumer: &Consumer) -> Vec<String> {
    let found = consumer.lint().into_iter().filter(|item| item.rule == RULE);
    let mut found: Vec<String> = found
        .map(|item| {
            let line = item.line.unwrap_or_default();
            let named = item
                .message
                .split(';')
                .next()
                .unwrap_or_default()
                .to_owned();
            format!("{path}:{line} {named}", path = item.path)
        })
        .collect();
    found.sort();
    found
}

fn uses(source: &str, entry: &str) -> String {
    format!("{source} uses the entry {entry} of its own directory or a directory above")
}

#[test]
fn own_code_that_uses_its_directory_entry_or_one_above_is_a_finding() {
    for (language, files, extension, entry) in [
        (Language::Rust, &RUST_JOBS[..], "rs", "src/jobs/mod.rs"),
        (
            Language::Python,
            &PYTHON_JOBS[..],
            "py",
            "src/jobs/__init__.py",
        ),
    ] {
        let expected = [
            uses(&format!("src/jobs/cleanup.{extension}:1"), entry),
            uses(&format!("src/jobs/guard/x.{extension}:1"), entry),
        ];
        assert_eq!(found(&measured(language, RULE, files)), expected);
    }
}

#[test]
fn entry_use_needs_an_enabled_directory_architecture_rule() {
    let files = [("src/lib.rs", "pub fn run() {}\n")];
    let consumer = measured(Language::Rust, RULE, &files);
    consumer.configure(
        "kind: directory-architecture\n",
        "kind: directory-architecture\n    enabled: false\n",
    );
    consumer.invalid();
}
