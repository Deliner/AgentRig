use review_runner::vcs::{FileKind, Kind, Repository};
use review_runner::{config, snapshot};
use std::{fs, os::unix::fs::PermissionsExt, path::Path, process::Command};

fn command(root: &Path, program: &str, args: &[&str]) {
    let output = Command::new(program)
        .args(args)
        .current_dir(root)
        .env("HGRCPATH", "")
        .env("HGRCSKIPREPO", "1")
        .env("HGPLAIN", "1")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn initialize(root: &Path, kind: Kind) {
    match kind {
        Kind::Git => {
            command(root, "git", &["init", "-q"]);
            command(
                root,
                "git",
                &["config", "user.email", "test@example.invalid"],
            );
            command(root, "git", &["config", "user.name", "Test"]);
        }
        Kind::Mercurial => command(root, "hg", &["init"]),
    }
}

fn commit(root: &Path, kind: Kind) -> String {
    let reference = match kind {
        Kind::Git => {
            command(root, "git", &["add", "-A"]);
            command(root, "git", &["commit", "-qm", "fixture"]);
            "HEAD"
        }
        Kind::Mercurial => {
            command(root, "hg", &["addremove"]);
            command(root, "hg", &["commit", "-m", "fixture", "-u", "Test"]);
            "."
        }
    };
    Repository::new(root, kind).resolve(reference).unwrap()
}

fn revisions(root: &Path, kind: Kind) -> (String, String) {
    initialize(root, kind);
    fs::write(root.join("old name"), b"before\n").unwrap();
    fs::write(root.join("binary"), [0, 255, 10]).unwrap();
    fs::write(root.join("executable"), b"run\n").unwrap();
    fs::set_permissions(root.join("executable"), fs::Permissions::from_mode(0o755)).unwrap();
    std::os::unix::fs::symlink("old name", root.join("link")).unwrap();
    let base = commit(root, kind);
    fs::rename(root.join("old name"), root.join("new name")).unwrap();
    fs::write(root.join("binary"), [0, 255, 20]).unwrap();
    let candidate = commit(root, kind);
    fs::write(root.join("new name"), b"uncommitted\n").unwrap();
    (base, candidate)
}

#[test]
fn native_backends_read_exact_revisions_and_preserve_worktrees() {
    for kind in [Kind::Git, Kind::Mercurial] {
        let root = tempfile::tempdir().unwrap();
        let (base, candidate) = revisions(root.path(), kind);
        let repository = Repository::new(root.path(), kind);
        assert_eq!(
            repository.read(&candidate, "new name").unwrap(),
            b"before\n"
        );
        assert_eq!(repository.read(&base, "binary").unwrap(), [0, 255, 10]);
        assert_eq!(repository.read(&candidate, "binary").unwrap(), [0, 255, 20]);
        assert_eq!(
            repository.changed_paths(&base, &candidate).unwrap(),
            ["binary", "new name", "old name"]
        );
        let tree = repository.tree(&candidate).unwrap();
        assert_eq!(tree["executable"].kind, FileKind::Executable);
        assert_eq!(tree["link"].kind, FileKind::Symlink);
        assert_eq!(tree["new name"].kind, FileKind::File);
        assert!(!tree["binary"].object.is_empty());
        let diff = repository.diff(&base, &candidate).unwrap();
        assert!(diff.contains("new name") && diff.contains("old name") && diff.contains("binary"));
        assert_eq!(
            fs::read(root.path().join("new name")).unwrap(),
            b"uncommitted\n"
        );
        assert!(repository.resolve("missing-reference").is_err());
    }
}

#[test]
fn mercurial_rejects_multiple_revisions_and_ignores_repository_commands() {
    let root = tempfile::tempdir().unwrap();
    let (_, candidate) = revisions(root.path(), Kind::Mercurial);
    fs::write(
        root.path().join(".hg/hgrc"),
        "[alias]\nlog = !touch executed\n[hooks]\npre-cat = touch executed\n",
    )
    .unwrap();
    let repository = Repository::new(root.path(), Kind::Mercurial);
    assert_eq!(repository.resolve(".").unwrap(), candidate);
    assert!(repository.resolve("all()").is_err());
    assert!(repository.read(&candidate, "binary").is_ok());
    assert!(!root.path().join("executed").exists());
}

fn snapshot_scope(kind: Kind) -> config::Repository {
    config::Repository {
        vcs: kind,
        visible_paths: vec!["*name".into(), "binary".into(), "executable".into()],
        contract_paths: vec!["new name".into()],
    }
}

#[test]
fn selected_backends_export_exact_snapshots_and_check_repair_boundaries() {
    for kind in [Kind::Git, Kind::Mercurial] {
        let root = tempfile::tempdir().unwrap();
        let (base, candidate) = revisions(root.path(), kind);
        let scope = snapshot_scope(kind);
        let output = root.path().join("exported");
        let result = snapshot::prepare(root.path(), (&base, &candidate), &scope, &output).unwrap();
        assert_eq!(result.candidate, candidate);
        assert_eq!(result.contract_paths, ["new name"]);
        assert_eq!(fs::read(output.join("binary")).unwrap(), [0, 255, 20]);
        assert_eq!(fs::read(output.join("new name")).unwrap(), b"before\n");
        assert_eq!(
            fs::metadata(output.join("executable"))
                .unwrap()
                .permissions()
                .mode()
                & 0o777,
            0o755
        );
        assert!(!output.join("link").exists());
        assert!(!output.join(".hg").exists());
        assert!(!output.join(".git").exists());
        assert_eq!(
            fs::read(root.path().join("new name")).unwrap(),
            b"uncommitted\n"
        );
        snapshot::check_boundary(root.path(), &base, &candidate, &scope).unwrap();
        let mut restricted = scope;
        restricted.visible_paths = vec!["new name".into(), "binary".into()];
        let error =
            snapshot::check_boundary(root.path(), &base, &candidate, &restricted).unwrap_err();
        assert!(
            error
                .to_string()
                .contains("outside visible_paths: old name")
        );
    }
}

#[test]
fn selected_backends_reject_symlinks_and_private_material() {
    for kind in [Kind::Git, Kind::Mercurial] {
        let root = tempfile::tempdir().unwrap();
        let exports = tempfile::tempdir().unwrap();
        let (_, candidate) = revisions(root.path(), kind);
        let mut scope = snapshot_scope(kind);
        scope.visible_paths.push("link".into());
        let output = exports.path().join("unsafe-link");
        let error =
            snapshot::prepare(root.path(), (&candidate, &candidate), &scope, &output).unwrap_err();
        assert!(error.to_string().contains("cannot expose symlink"));
        fs::write(root.path().join("binary"), b"-----BEGIN PRIVATE KEY-----").unwrap();
        let secret = commit(root.path(), kind);
        let output = exports.path().join("unsafe-key");
        let error = snapshot::prepare(
            root.path(),
            (&secret, &secret),
            &snapshot_scope(kind),
            &output,
        )
        .unwrap_err();
        assert!(error.to_string().contains("private key material"));
    }
    assert!(snapshot::safe_path("nested/.hg/hgrc").is_err());
}

#[test]
fn repository_yaml_selects_backend_and_rejects_unknown_systems() {
    let yaml = "vcs: mercurial\nvisible_paths: ['src/**']\ncontract_paths: []\n";
    let scope: config::Repository = config::yaml::decode(yaml).unwrap();
    assert!(matches!(scope.vcs, Kind::Mercurial));
    let invalid = yaml.replace("mercurial", "unknown");
    assert!(config::yaml::decode::<config::Repository>(&invalid).is_err());
}

#[test]
fn inventory_discovery_does_not_guess_between_two_repositories() {
    let root = tempfile::tempdir().unwrap();
    assert!(Repository::discover(root.path()).unwrap().is_none());
    fs::create_dir(root.path().join(".git")).unwrap();
    assert!(Repository::discover(root.path()).unwrap().is_some());
    fs::create_dir(root.path().join(".hg")).unwrap();
    assert!(Repository::discover(root.path()).is_err());
    fs::remove_dir(root.path().join(".git")).unwrap();
    fs::write(root.path().join(".hg/requires"), "unknown-test-format\n").unwrap();
    let repository = Repository::discover(root.path()).unwrap().unwrap();
    assert!(repository.working_files().is_err());
    assert!(repository.head().is_err());
}

#[test]
fn native_heads_distinguish_unborn_repositories_from_committed_revisions() {
    for kind in [Kind::Git, Kind::Mercurial] {
        let root = tempfile::tempdir().unwrap();
        initialize(root.path(), kind);
        let repository = Repository::new(root.path(), kind);
        assert_eq!(repository.head().unwrap(), None);
        fs::write(root.path().join("value"), "committed").unwrap();
        let revision = commit(root.path(), kind);
        assert_eq!(repository.head().unwrap(), Some(revision));
    }
}

#[test]
fn full_revision_exports_preserve_project_inputs_and_native_file_kinds() {
    for kind in [Kind::Git, Kind::Mercurial] {
        let root = tempfile::tempdir().unwrap();
        let (base, _) = revisions(root.path(), kind);
        fs::create_dir_all(root.path().join(".agents/skills/example")).unwrap();
        fs::write(
            root.path().join(".agents/skills/example/SKILL.md"),
            "instructions",
        )
        .unwrap();
        fs::write(root.path().join("agentrig.yaml"), "version: 1\n").unwrap();
        let candidate = commit(root.path(), kind);
        fs::write(root.path().join("agentrig.yaml"), "dirty").unwrap();
        let repository = Repository::new(root.path(), kind);
        let export = repository.export_revision(&candidate).unwrap();
        assert_eq!(
            fs::read(export.path().join("agentrig.yaml")).unwrap(),
            b"version: 1\n"
        );
        assert_eq!(
            fs::read(export.path().join(".agents/skills/example/SKILL.md")).unwrap(),
            b"instructions"
        );
        assert_exported_file_kinds(export.path());
        assert!(!export.path().join("old name").exists());
        assert!(!export.path().join(".git").exists());
        assert!(!export.path().join(".hg").exists());
        let previous = repository.export_revision(&base).unwrap();
        assert_eq!(fs::read(previous.path().join("link")).unwrap(), b"before\n");
        assert!(!previous.path().join("new name").exists());
        assert_eq!(
            fs::read(root.path().join("agentrig.yaml")).unwrap(),
            b"dirty"
        );
    }
}

fn assert_exported_file_kinds(root: &Path) {
    assert_eq!(fs::read(root.join("binary")).unwrap(), [0, 255, 20]);
    assert_eq!(
        fs::read_link(root.join("link")).unwrap(),
        Path::new("old name")
    );
    assert_eq!(
        fs::metadata(root.join("executable"))
            .unwrap()
            .permissions()
            .mode()
            & 0o777,
        0o755
    );
}

#[test]
fn index_export_preserves_staged_content_and_reports_unsupported_backends() {
    let root = tempfile::tempdir().unwrap();
    revisions(root.path(), Kind::Git);
    fs::write(root.path().join("new name"), "staged").unwrap();
    command(root.path(), "git", &["add", "new name"]);
    fs::write(root.path().join("new name"), "unstaged").unwrap();
    let repository = Repository::new(root.path(), Kind::Git);
    let before = repository.index_entries().unwrap();
    let export = repository.export_index().unwrap();
    assert_eq!(fs::read(export.path().join("new name")).unwrap(), b"staged");
    assert_eq!(fs::read(root.path().join("new name")).unwrap(), b"unstaged");
    assert_eq!(repository.index_entries().unwrap(), before);
    let mercurial = tempfile::tempdir().unwrap();
    initialize(mercurial.path(), Kind::Mercurial);
    let error = Repository::new(mercurial.path(), Kind::Mercurial)
        .export_index()
        .unwrap_err();
    assert!(error.to_string().contains("Mercurial has no staging index"));
}

#[test]
fn full_revision_export_rejects_control_metadata_and_submodules() {
    let root = tempfile::tempdir().unwrap();
    let (_, revision) = revisions(root.path(), Kind::Git);
    command(
        root.path(),
        "git",
        &[
            "update-index",
            "--add",
            "--cacheinfo",
            &format!("160000,{revision},dependency"),
        ],
    );
    command(root.path(), "git", &["commit", "-qm", "submodule"]);
    let repository = Repository::new(root.path(), Kind::Git);
    let error = repository.export_revision("HEAD").unwrap_err();
    assert!(
        error
            .to_string()
            .contains("cannot include submodule dependency")
    );
    command(
        root.path(),
        "git",
        &["update-index", "--force-remove", "dependency"],
    );
    fs::create_dir(root.path().join(".hg")).unwrap();
    fs::write(root.path().join(".hg/hgrc"), "[hooks]\n").unwrap();
    let candidate = commit(root.path(), Kind::Git);
    let error = repository.export_revision(&candidate).unwrap_err();
    assert!(
        error
            .to_string()
            .contains("unsafe VCS export path: .hg/hgrc")
    );
}
