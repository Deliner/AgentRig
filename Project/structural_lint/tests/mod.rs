// DECISION: D036
use super::{config::Config, evaluation::evaluate};
use std::fs;

mod architecture;
mod diagnostics;

fn findings(files: &[(&str, &str)], rule: &str) -> Vec<(String, Option<usize>, Option<usize>)> {
    let root = tempfile::tempdir().unwrap();
    for (path, content) in files {
        let path = root.path().join(path);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, content).unwrap();
    }
    let source = format!(
        "version: 1\nconfig_skill: s\nrules:\n- {{{rule}, warning_skill: s, error_skill: s}}\n"
    );
    let config: Config = crate::shared::yaml::decode(&source).unwrap();
    evaluate(root.path(), &config)
        .unwrap()
        .into_iter()
        .map(|item| (item.path, item.line, item.actual))
        .collect()
}

#[test]
fn line_width_reports_each_line_over_the_limit_in_characters() {
    let wide = "x".repeat(121);
    let accented = "é".repeat(120);
    let widest = "y".repeat(130);
    let readme = format!("short\n{wide}\n{accented}\n{widest}\n");
    let rule = "id: width, kind: line-width, target: file, include: ['**/README.md'], error: 120";
    let found = findings(&[("docs/README.md", &readme), ("notes.md", &wide)], rule);
    assert_eq!(
        found,
        [
            ("docs/README.md".into(), Some(2), Some(121)),
            ("docs/README.md".into(), Some(4), Some(130)),
        ]
    );
}

// DECISION: D072
#[test]
fn dump_name_reports_only_generic_dumping_ground_directories() {
    let files = [
        ("Project/feature/utils/mod.rs", "fn main() {}"),
        ("Project/feature/shared/mod.rs", "fn main() {}"),
        ("Project/feature/parser/mod.rs", "fn main() {}"),
        ("Project/helpers/mod.rs", "fn main() {}"),
    ];
    let rule =
        "id: dump, kind: dump-name, target: directory, include: ['Project/**'], level: error";
    let mut found = findings(&files, rule);
    found.sort();
    assert_eq!(
        found,
        [
            ("Project/feature/utils".into(), None, Some(1)),
            ("Project/helpers".into(), None, Some(1)),
        ]
    );
}

#[test]
fn paired_calls_checks_only_the_pairs_its_rule_configures() {
    let source = "fn save(db: &mut Db, file: &mut File) -> Result<()> {
    db.begin();
    write()?;
    db.commit();
    file.open();
    write()?;
    file.close();
    Ok(())
}
";
    let rule = "id: pairs, kind: paired-calls, target: file, include: ['**'], extensions: [.rs], \
        level: error, pairs: [{open: open, close: close}]";
    let found = findings(&[("src/save.rs", source)], rule);
    assert_eq!(found, [("src/save.rs".into(), Some(5), Some(1))]);
}

#[test]
fn forbidden_file_reports_every_selected_file() {
    let files = [
        ("Project/README.md", ""),
        ("Project/notes.md", "text"),
        ("Project/feature/guide.md", ""),
        ("Project/feature/skills/check/SKILL.md", "skill"),
        ("Project/feature/code.rs", "fn main() {}"),
        ("outside.md", "text"),
    ];
    let rule = "id: placement, kind: forbidden-file, target: file, include: ['Project/**'], \
        extensions: [.md], exclude: ['**/README.md', '**/SKILL.md'], level: error";
    let mut found = findings(&files, rule);
    found.sort();
    assert_eq!(
        found,
        [
            ("Project/feature/guide.md".into(), None, Some(1)),
            ("Project/notes.md".into(), None, Some(1)),
        ]
    );
}
