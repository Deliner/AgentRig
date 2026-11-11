use super::{Consumer, Language, SKILL};
use crate::structural_lint::Mode;

const INVALID: [(&str, &str); 5] = [
    ("extensions: [.py]", "extensions: [.go]"),
    ("architecture:", "architecture:\n      unknown: true"),
    ("rust_roots: []", "rust_roots: ['../lib.rs']"),
    ("extensions: [.py]", "extensions: []"),
    ("level: error", "level: error\n    error: 4"),
];

#[test]
fn architecture_config_check_rejects_invalid_settings() {
    for (from, to) in INVALID {
        let consumer = Consumer::new(Language::Python);
        consumer.configure(from, to);
        consumer.invalid();
    }
}

#[test]
fn enabled_rust_rules_require_crate_roots() {
    for enabled in [true, false] {
        let consumer = Consumer::new(Language::Rust);
        consumer.configure("rust_roots: [src/lib.rs]", "rust_roots: []");
        let last = format!("error_skill: {SKILL}\n");
        consumer.configure(&last, &format!("{last}    enabled: {enabled}\n"));
        let checked = consumer.analysis(Mode::ConfigCheck).is_ok();
        let linted = consumer.analysis(Mode::Lint);
        let clean = linted.as_ref().map(Vec::is_empty).ok();
        let expected = (!enabled, (!enabled).then_some(true));
        assert_eq!((checked, clean), expected);
    }
}
