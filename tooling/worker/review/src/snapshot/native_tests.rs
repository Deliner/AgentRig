#[path = "../vcs/tests/native_fixture.rs"]
pub mod native_fixture;
use native_fixture::{commit, revisions};
use review_runner::{config, snapshot, vcs::Kind};
use std::{fs, os::unix::fs::PermissionsExt};

fn snapshot_scope(kind: Kind) -> config::Repository {
    config::Repository {
        vcs: kind.into(),
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
