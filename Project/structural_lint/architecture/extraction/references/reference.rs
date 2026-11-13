use crate::structural_lint::architecture::Relation;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Target {
    RustPath {
        path: String,
        scope: Vec<String>,
    },
    RustMacro(RustMacro),
    RustDerive {
        path: String,
        scope: Vec<String>,
    },
    RustModule {
        path: Vec<String>,
        inline: bool,
        file: Option<String>,
    },
    // DECISION: D136
    RustInclude {
        file: String,
        scope: Vec<String>,
    },
    PythonModule(String),
    PythonFrom {
        module: String,
        names: Vec<String>,
    },
}
impl Target {
    pub fn relation(&self) -> Relation {
        match self {
            Self::RustModule { .. } | Self::RustInclude { .. } => Relation::Declaration,
            Self::RustPath { .. }
            | Self::RustMacro(_)
            | Self::RustDerive { .. }
            | Self::PythonModule(_)
            | Self::PythonFrom { .. } => Relation::Use,
        }
    }
    pub fn expansion(&self) -> bool {
        match self {
            Self::RustMacro(_) | Self::RustDerive { .. } => true,
            Self::RustPath { .. }
            | Self::RustModule { .. }
            | Self::RustInclude { .. }
            | Self::PythonModule(_)
            | Self::PythonFrom { .. } => false,
        }
    }
    pub(super) fn repeatable(&self) -> bool {
        match self {
            Self::RustModule { .. } => true,
            Self::RustPath { .. }
            | Self::RustMacro(_)
            | Self::RustDerive { .. }
            | Self::RustInclude { .. }
            | Self::PythonModule(_)
            | Self::PythonFrom { .. } => false,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RustMacro {
    pub path: String,
    pub scope: Vec<String>,
    pub literal: Option<String>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Reference {
    pub line: usize,
    pub target: Target,
    pub test: bool,
}
