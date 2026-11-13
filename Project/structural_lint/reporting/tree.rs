use super::explain::selected_path;
use super::structure::{self, Texts};
use crate::shared::paths::glob_patterns::globs;
use crate::shared::{console, fs_with_context as fs};
use crate::structural_lint::Location;
use crate::structural_lint::rule_identities::descriptor::TEST_CODE;
use crate::structural_lint::targeting::file_discovery;
use anyhow::{Result, bail, ensure};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

const UNMEASURED: [&str; 2] = ["**/.*", "**/target"];

pub(super) fn run(location: Location<'_>, args: &[String]) -> Result<i32> {
    let [path] = args else {
        bail!("lint-tree PATH");
    };
    let path = selected_path(path)?;
    let config = location.load()?;
    let inventory = file_discovery::collect(location.root, &globs(&config.exclude)?)?;
    ensure!(
        inventory.directories.contains_key(&path),
        "lint-tree PATH must name a directory of the lint inventory: {}",
        path.display()
    );
    console::stdout(render(&inventory.directories, &path));
    Ok(0)
}

pub fn lint_structure(args: &[String]) -> Result<i32> {
    let [path] = args else {
        bail!("lint-structure PATH");
    };
    let root = fs::canonicalize(path)?;
    ensure!(
        fs::metadata(&root)?.is_dir(),
        "lint-structure PATH must name a directory: {path}"
    );
    let inventory = file_discovery::unversioned(&root, &globs(&UNMEASURED.map(String::from))?)?;
    let tests = globs(&TEST_CODE.iter().map(|&p| p.into()).collect::<Vec<_>>())?;
    let mut texts = Texts::default();
    for file in &inventory.files {
        let contract = file
            .file_name()
            .is_some_and(|name| name == "architecture.yaml");
        let rust = file.extension().is_some_and(|extension| extension == "rs");
        let production = rust && !tests.is_match(file);
        if contract {
            let directory = file.parent().unwrap_or(Path::new("")).to_path_buf();
            texts
                .contracts
                .insert(directory, fs::read_to_string(root.join(file))?);
        } else if production {
            texts
                .code
                .insert(file.clone(), fs::read_to_string(root.join(file))?);
        }
    }
    console::stdout(structure::report(&texts)?);
    Ok(0)
}

fn render(directories: &BTreeMap<PathBuf, BTreeSet<String>>, path: &Path) -> String {
    let mut lines = vec![path.display().to_string()];
    branch(directories, path, "", &mut lines);
    lines.join("\n")
}

fn branch(
    directories: &BTreeMap<PathBuf, BTreeSet<String>>,
    directory: &Path,
    indent: &str,
    lines: &mut Vec<String>,
) {
    let Some(entries) = directories.get(directory) else {
        return;
    };
    let count = entries.len();
    let root = directory == Path::new(".");
    for (index, name) in entries.iter().enumerate() {
        let last = index + 1 == count;
        let (joint, continuation) = if last {
            ("└── ", "    ")
        } else {
            ("├── ", "│   ")
        };
        lines.push(format!("{indent}{joint}{name}"));
        let child = if root {
            PathBuf::from(name)
        } else {
            directory.join(name)
        };
        branch(
            directories,
            &child,
            &format!("{indent}{continuation}"),
            lines,
        );
    }
}
