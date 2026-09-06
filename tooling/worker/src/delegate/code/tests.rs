use super::*;
use std::os::unix::fs::symlink;

#[test]
fn isolated_patch_preserves_original_and_applies_to_snapshot() -> Result<()> {
    let root = tempfile::tempdir()?;
    let snapshot = root.path().join("snapshot");
    fs::create_dir(&snapshot)?;
    fs::write(snapshot.join("value.txt"), "before\n")?;
    let workspace = Workspace::prepare(&snapshot, &root.path().join("code"))?;
    fs::write(workspace.project().join("value.txt"), "after\n")?;
    let (patch, report) = workspace.patch(&["value.txt".into()])?;
    assert_eq!(report["changed_paths"], json!(["value.txt"]));
    assert_eq!(fs::read_to_string(snapshot.join("value.txt"))?, "before\n");
    assert!(!workspace.project().join(".git").exists());
    let patch_path = root.path().join("result.patch");
    fs::write(&patch_path, &patch)?;
    let applied = Command::new("/usr/bin/git")
        .args(["apply", "--check"])
        .arg(&patch_path)
        .current_dir(&snapshot)
        .output()?;
    assert!(
        applied.status.success(),
        "{}",
        String::from_utf8_lossy(&applied.stderr)
    );
    Ok(())
}

#[test]
fn code_scope_and_untrusted_files_are_rejected() -> Result<()> {
    let root = tempfile::tempdir()?;
    let snapshot = root.path().join("snapshot");
    fs::create_dir(&snapshot)?;
    fs::write(snapshot.join("value.txt"), "before\n")?;
    let workspace = Workspace::prepare(&snapshot, &root.path().join("code"))?;
    fs::write(workspace.project().join("other.txt"), "new\n")?;
    assert!(
        workspace
            .patch(&["value.txt".into()])
            .unwrap_err()
            .to_string()
            .contains("write_paths")
    );
    symlink("/etc/passwd", workspace.project().join("link"))?;
    assert!(workspace.patch(&["**".into()]).is_err());
    Ok(())
}

#[test]
fn binary_patch_preserves_bytes_and_executable_mode() -> Result<()> {
    let root = tempfile::tempdir()?;
    let snapshot = root.path().join("snapshot");
    fs::create_dir(&snapshot)?;
    fs::write(snapshot.join("value.bin"), [0, 255, 1])?;
    let workspace = Workspace::prepare(&snapshot, &root.path().join("code"))?;
    let changed = workspace.project().join("value.bin");
    fs::write(&changed, [0, 254, 2, 3])?;
    fs::set_permissions(&changed, fs::Permissions::from_mode(0o755))?;
    let (patch, _) = workspace.patch(&["value.bin".into()])?;
    let path = root.path().join("result.patch");
    fs::write(&path, patch)?;
    let applied = Command::new("/usr/bin/git")
        .arg("apply")
        .arg(path)
        .current_dir(&snapshot)
        .output()?;
    assert!(
        applied.status.success(),
        "{}",
        String::from_utf8_lossy(&applied.stderr)
    );
    assert_eq!(fs::read(snapshot.join("value.bin"))?, [0, 254, 2, 3]);
    assert_ne!(
        fs::metadata(snapshot.join("value.bin"))?
            .permissions()
            .mode()
            & 0o111,
        0
    );
    Ok(())
}
