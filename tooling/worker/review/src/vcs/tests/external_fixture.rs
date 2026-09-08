use review_runner::vcs::{Kind, Repository, external::Adapter};
use std::{
    collections::BTreeMap,
    fs,
    os::unix::ffi::OsStrExt,
    path::{Path, PathBuf},
    process::Command,
};

pub fn example() -> Adapter {
    // Cached tests can outlive the exported source tree where they were compiled.
    let script = PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR").unwrap())
        .join("../examples/external_vcs.py");
    Adapter {
        command: vec![
            "python3".into(),
            "-B".into(),
            script.to_str().unwrap().into(),
        ],
    }
}

pub fn hg(root: &Path, args: &[&str]) {
    let output = Command::new("hg")
        .args(args)
        .current_dir(root)
        .env("HGPLAIN", "1")
        .env("HGRCPATH", "")
        .env("HGRCSKIPREPO", "1")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

pub fn commit(root: &Path) -> String {
    hg(root, &["addremove"]);
    hg(root, &["commit", "-m", "fixture", "-u", "Test"]);
    Repository::new(root, Kind::Mercurial).resolve(".").unwrap()
}

pub fn contents(root: &Path) -> BTreeMap<PathBuf, Vec<u8>> {
    let mut files = BTreeMap::new();
    for entry in fs::read_dir(root).unwrap() {
        let entry = entry.unwrap();
        let kind = entry.file_type().unwrap();
        let directory = kind.is_dir();
        if directory {
            files.extend(contents(&entry.path()));
            continue;
        }
        let symlink = kind.is_symlink();
        let bytes = if symlink {
            fs::read_link(entry.path())
                .unwrap()
                .as_os_str()
                .as_bytes()
                .to_vec()
        } else {
            fs::read(entry.path()).unwrap()
        };
        files.insert(entry.path(), bytes);
    }
    files
}

pub fn revisions(root: &Path) -> (String, String) {
    hg(root, &["init"]);
    let adapter = example();
    assert_eq!(adapter.head(root).unwrap(), None);
    fs::write(root.join("binary"), [0, 255, 10]).unwrap();
    fs::write(root.join("old name"), "before").unwrap();
    std::os::unix::fs::symlink("old name", root.join("link")).unwrap();
    let base = commit(root);
    fs::rename(root.join("old name"), root.join("new name")).unwrap();
    fs::write(root.join("binary"), [0, 255, 20]).unwrap();
    let candidate = commit(root);
    fs::write(root.join("new name"), "uncommitted").unwrap();
    (base, candidate)
}
