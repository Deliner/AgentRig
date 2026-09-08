#[path = "../vcs/tests/external_fixture.rs"]
pub mod external_fixture;
use external_fixture::{contents, example, revisions};
use review_runner::{config, snapshot, vcs::Backend};
use std::fs;

#[test]
fn configured_external_source_uses_shared_snapshot_visibility_and_file_restrictions() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path();
    let (base, candidate) = revisions(root);
    let output = tempfile::tempdir().unwrap();
    let mut scope = config::Repository {
        vcs: Backend::External(example()),
        visible_paths: vec!["binary".into(), "*name".into()],
        contract_paths: vec!["new name".into()],
    };
    let before = contents(root);
    let result = snapshot::prepare(
        root,
        (&base, &candidate),
        &scope,
        &output.path().join("valid"),
    )
    .unwrap();
    assert_eq!(result.candidate, candidate);
    assert_eq!(
        fs::read(output.path().join("valid/new name")).unwrap(),
        b"before"
    );
    scope.visible_paths = vec!["binary".into()];
    assert!(snapshot::check_boundary(root, &base, &candidate, &scope).is_err());
    scope.visible_paths = vec!["**".into()];
    let error = snapshot::prepare(
        root,
        (&base, &candidate),
        &scope,
        &output.path().join("symlink"),
    )
    .unwrap_err();
    assert!(error.to_string().contains("symlink or submodule"));
    assert_eq!(contents(root), before);
}
