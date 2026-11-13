use std::{
    fs,
    path::{Path, PathBuf},
};

mod clippy;
mod coverage;
mod defaults;
mod delivery;
mod doctor;
mod runtime;
mod toolchain;

fn write(root: &Path, path: &str, text: &str) {
    let path = root.join(path);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, text).unwrap();
}

fn installed_runtime(root: &Path) -> PathBuf {
    let mut runtimes = fs::read_dir(root.join(".agentrig/bin"))
        .unwrap()
        .map(|entry| entry.unwrap().path().join("agentrig"))
        .filter(|path| path.is_file());
    let runtime = runtimes.next().unwrap();
    assert_eq!(runtimes.next(), None);
    runtime
}
