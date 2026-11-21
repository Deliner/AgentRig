use std::path::PathBuf;

#[derive(Clone, Debug)]
pub struct Dependency {
    pub source: PathBuf,
    pub target: PathBuf,
    pub line: usize,
    pub relation: Relation,
    pub test: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Relation {
    Declaration,
    Use,
    // DECISION: D227
    Initialization,
}
impl Relation {
    pub fn declares(self) -> bool {
        match self {
            Self::Declaration => true,
            Self::Use | Self::Initialization => false,
        }
    }
    // DECISION: D227
    pub fn uses(self) -> bool {
        match self {
            Self::Use => true,
            Self::Declaration | Self::Initialization => false,
        }
    }
}
