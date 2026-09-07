use super::{Backend, Kind, export};
use anyhow::{Result, ensure};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, path::Path};

#[derive(Serialize)]
pub struct Generation<'a> {
    pub binary: &'a str,
    pub directory: &'a str,
    pub ignored: &'a [String],
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Generated {
    pub root_command: String,
    pub files: BTreeMap<String, String>,
}

impl Backend {
    pub fn generate(&self, root: &Path, request: &Generation<'_>) -> Result<Generated> {
        let generated = match self {
            Self::Native(kind) => request.native(*kind),
            Self::External(adapter) => {
                adapter.call(root, "generate", serde_json::to_value(request)?)?
            }
        };
        ensure!(
            !generated.root_command.trim().is_empty() && !generated.root_command.contains('\0'),
            "VCS root command must be nonempty and NUL-free"
        );
        for path in generated.files.keys() {
            export::validate_path(path)?;
            let hook = Path::new(path).starts_with(request.directory) && path != request.directory;
            let auxiliary = path.starts_with(&format!("{}.", request.directory))
                && Path::new(path).parent() == Path::new(request.directory).parent();
            ensure!(
                hook || auxiliary,
                "generated VCS file is outside its hook area: {path}"
            );
        }
        Ok(generated)
    }
}

impl Generation<'_> {
    fn native(&self, kind: Kind) -> Generated {
        let mut files: BTreeMap<_, _> = kind
            .hooks(self.binary)
            .into_iter()
            .map(|(name, contents)| (format!("{}/{name}", self.directory), contents))
            .collect();
        let mercurial = kind == Kind::Mercurial;
        if mercurial {
            let patterns = self
                .ignored
                .iter()
                .map(|path| format!("{path}/**\n"))
                .collect::<String>();
            files.insert(
                format!("{}.hgignore", self.directory),
                format!("syntax: glob\n{patterns}"),
            );
        }
        Generated {
            root_command: kind.root_command().into(),
            files,
        }
    }
}
