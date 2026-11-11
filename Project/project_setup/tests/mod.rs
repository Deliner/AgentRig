use std::{
    fs,
    path::{Path, PathBuf},
};

mod defaults;
mod delivery;
mod doctor;
mod runtime;

fn installed_runtime(root: &Path) -> PathBuf {
    let mut runtimes = fs::read_dir(root.join(".agentrig/bin"))
        .unwrap()
        .map(|entry| entry.unwrap().path().join("agentrig"))
        .filter(|path| path.is_file());
    let runtime = runtimes.next().unwrap();
    assert_eq!(runtimes.next(), None);
    runtime
}
