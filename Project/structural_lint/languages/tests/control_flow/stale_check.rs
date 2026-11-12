use super::measured;
use crate::structural_lint::rules::control_flow::ControlFlow;

// DECISION: D171
#[test]
fn rust_flags_a_check_whose_propagated_action_depends_on_a_normal_absence() {
    let source = "fn load(path: &Path, root: PathBuf, lock: PathBuf) -> Result<Option<String>> {
    if path.exists() {
        let text = fs::read_to_string(path)?;
        return Ok(Some(text));
    }
    if !root.is_dir() {
        return Ok(None);
    }
    for entry in fs::read_dir(&root)? {
        entry?;
    }
    let missing = !lock.as_path().is_file();
    if missing {
        log(\"no lock\");
    } else {
        let file = File::open(&lock).context(\"open lock\")?;
    }
    Ok(None)
}
async fn sync(target: PathBuf) -> Result<()> {
    let ready = target.exists();
    if !ready {
        return Ok(());
    }
    tokio::fs::remove_file(&target).await?;
    Ok(())
}
";
    assert_eq!(
        measured(source, ControlFlow::StaleCheck),
        [
            (2, "exists".to_string()),
            (6, "is_dir".to_string()),
            (12, "is_file".to_string()),
            (21, "exists".to_string()),
        ]
    );
}

#[test]
fn rust_skips_checks_with_a_failing_absence_or_without_a_governing_if() {
    let source = "fn skip(path: &Path, data: PathBuf, cache: PathBuf) -> Result<()> {
    if !path.exists() {
        fs::create_dir_all(path)?;
    }
    if data.is_file() {
        fs::read(&data)?;
    } else {
        return Err(missing());
    }
    let present = cache.exists();
    fs::read(&cache)?;
    Ok(())
}
fn unguarded(path: &Path) -> Result<Vec<u8>> {
    expect_present(path.exists());
    Ok(fs::read(path)?)
}
";
    assert!(measured(source, ControlFlow::StaleCheck).is_empty());
}

#[test]
fn rust_skips_unpropagated_independent_other_path_justified_and_test_checks() {
    let source = "fn skip(other: &Path, dir: PathBuf, log: PathBuf) -> Result<()> {
    if other.exists() {
        let _ = fs::remove_file(other);
    }
    if dir.is_dir() {
        note(\"dir\");
    }
    fs::remove_dir_all(&dir)?;
    if log.exists() {
        fs::read(&dir)?;
    }
    // DECISION: D001
    if log.exists() {
        fs::read(&log)?;
    }
    Ok(())
}
#[cfg(test)]
mod tests {
    fn check(path: &Path) -> Result<()> {
        if path.exists() {
            fs::read(path)?;
        }
        Ok(())
    }
}
";
    assert!(measured(source, ControlFlow::StaleCheck).is_empty());
}
