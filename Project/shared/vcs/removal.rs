// DECISION: D233
use crate::shared::fs_with_context as fs;
use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
};

const IGNORE_FILE: &str = ".gitignore";

pub struct Removed {
    pub path: PathBuf,
    pub bytes: u64,
}

pub struct Removal {
    pub removed: Vec<Removed>,
    pub failures: Vec<String>,
}

pub fn remove(entries: &[PathBuf], kept: &dyn Fn(&Path) -> bool) -> Removal {
    let mut sweep = Sweep {
        kept,
        removal: Removal {
            removed: Vec::new(),
            failures: Vec::new(),
        },
    };
    for entry in entries {
        let guarding = ignore_file(entry);
        if guarding {
            continue;
        }
        if let Cleared::Gone(bytes) = sweep.clear(entry) {
            let path = entry.clone();
            sweep.removal.removed.push(Removed { path, bytes });
        }
    }
    sweep.removal
}

enum Cleared {
    Gone(u64),
    Remains,
}

struct Sweep<'a> {
    kept: &'a dyn Fn(&Path) -> bool,
    removal: Removal,
}

impl Sweep<'_> {
    fn clear(&mut self, path: &Path) -> Cleared {
        let kept = (self.kept)(path);
        if kept {
            return Cleared::Remains;
        }
        match fs::present(fs::symlink_metadata(path)) {
            Ok(Some(metadata)) if metadata.is_dir() => self.directory(path),
            Ok(Some(metadata)) => self.settle(fs::remove_file(path), metadata.len()),
            Ok(None) => Cleared::Gone(0),
            Err(error) => self.fail(error),
        }
    }

    fn directory(&mut self, path: &Path) -> Cleared {
        let children = match listed(path) {
            Ok(children) => children,
            Err(error) => return self.fail(error),
        };
        let (guards, others): (Vec<PathBuf>, Vec<PathBuf>) =
            children.into_iter().partition(|child| ignore_file(child));
        let mut cleared = Vec::new();
        let mut remains = false;
        for path in others {
            match self.clear(&path) {
                Cleared::Gone(bytes) => cleared.push(Removed { path, bytes }),
                Cleared::Remains => remains = true,
            }
        }
        let freed: u64 = cleared.iter().map(|removed| removed.bytes).sum();
        let outcome = if remains {
            Cleared::Remains
        } else {
            self.emptied(path, &guards)
        };
        match outcome {
            Cleared::Gone(bytes) => Cleared::Gone(freed + bytes),
            Cleared::Remains => {
                self.removal.removed.extend(cleared);
                Cleared::Remains
            }
        }
    }

    fn emptied(&mut self, path: &Path, guards: &[PathBuf]) -> Cleared {
        let mut bytes = 0_u64;
        for guard in guards {
            let Cleared::Gone(freed) = self.clear(guard) else {
                return Cleared::Remains;
            };
            bytes += freed;
        }
        self.settle(fs::remove_dir(path), bytes)
    }

    fn settle(&mut self, removal: fs::Result<()>, bytes: u64) -> Cleared {
        match removal {
            Ok(()) => Cleared::Gone(bytes),
            Err(error) => self.fail(error),
        }
    }

    fn fail(&mut self, error: fs::Error) -> Cleared {
        let failure = format!("{:#}", anyhow::Error::new(error));
        self.removal.failures.push(failure);
        Cleared::Remains
    }
}

fn ignore_file(path: &Path) -> bool {
    path.file_name() == Some(OsStr::new(IGNORE_FILE))
}

fn listed(path: &Path) -> fs::Result<Vec<PathBuf>> {
    fs::read_dir(path)?.map(|entry| Ok(entry?.path())).collect()
}
