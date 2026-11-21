use crate::shared::fs_with_context as fs;
use crate::structural_lint::architecture::{Python, Target};
use crate::structural_lint::languages::{Imported, Project, PythonImport, Source};
use crate::structural_lint::rule_identities::Settings;
use std::path::Path;

// DECISION: D229
pub(super) struct Declared<'a> {
    pub(super) settings: &'a Settings,
    pub(super) root: &'a Path,
    pub(super) python: Python<'a>,
}
impl Project for Declared<'_> {
    fn foreign_crate(&self, name: &str) -> bool {
        self.settings.external.rust.contains(name)
    }
    fn python_import(&self, from: &Path, import: PythonImport<'_>) -> Imported {
        let target = Target::PythonFrom {
            module: import.module.to_owned(),
            names: vec![import.name.to_owned()],
        };
        let Ok(resolved) = self.python.resolve(from, &target) else {
            return Imported::Unknown;
        };
        let outside = resolved.external.is_some();
        if outside {
            return Imported::Outside;
        }
        let read: anyhow::Result<Vec<Source>> = resolved
            .files
            .into_iter()
            .map(|path| {
                let text = fs::read_to_string(self.root.join(&path))?;
                Ok(Source { path, text })
            })
            .collect();
        match read {
            Ok(sources) => Imported::Inside(sources),
            Err(_) => Imported::Unknown,
        }
    }
}
