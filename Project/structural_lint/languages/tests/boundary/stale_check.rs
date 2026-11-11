use super::measured;
use crate::structural_lint::rules::reference::Reference;

#[test]
fn rust_flags_a_check_before_a_filesystem_action_on_the_same_path() {
    let source = r#"fn prepare(dir: &Path, file: &Path, config: &Config) -> io::Result<()> {
    if !dir.exists() {
        fs::create_dir_all(dir)?;
    }
    if file.is_file() {
        std::fs::remove_file(&file)?;
    }
    if config.path.as_path().is_dir() {
        let entries = (0..1).map(|_| fs::read_dir(&config.path));
    }
    if Path::new("lock").exists() {
        let lock = File::open(Path::new("lock"));
    }
    Ok(())
}
"#;
    assert_eq!(
        measured(source, Reference::StaleCheck),
        [
            (2, "exists".to_string()),
            (5, "is_file".to_string()),
            (8, "is_dir".to_string()),
            (11, "exists".to_string())
        ]
    );
}

#[test]
fn rust_skips_earlier_actions_other_paths_nested_functions_and_other_checks() {
    let source = r#"fn before(path: &Path) {
    fs::remove_file(path);
    if path.exists() {
        log(path);
    }
}
fn other(path: &Path, other: &Path) {
    if other.exists() {
        fs::remove_file(path);
    }
}
fn nested(path: &Path) {
    if path.is_dir() {
        fn inner(path: &Path) {
            fs::remove_dir(path);
        }
    }
}
fn metadata(path: &Path) {
    if path.metadata().is_ok() {
        fs::read(path);
    }
}
"#;
    assert!(measured(source, Reference::StaleCheck).is_empty());
}

#[test]
fn rust_skips_a_justified_check_and_test_code() {
    let source = r#"fn read(path: &Path) {
    // DECISION: D001
    if path.is_file() {
        fs::read(path);
    }
}
#[cfg(test)]
mod tests {
    fn clean(path: &Path) {
        if path.exists() {
            fs::remove_file(path);
        }
    }
}
"#;
    assert!(measured(source, Reference::StaleCheck).is_empty());
}
