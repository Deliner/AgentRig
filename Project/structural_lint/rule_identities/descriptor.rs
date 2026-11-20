use super::kind::Kind;
use super::parameters::Parameters;

pub struct Descriptor {
    pub default_id: &'static str,
    pub exclude: &'static [&'static str],
    pub target: &'static str,
    pub metric: &'static str,
    pub skill: &'static str,
    pub parameters: Parameters,
    // DECISION: D222
    pub accepts_exception: bool,
}
// DECISION: D223
pub(crate) const TEST_CODE: &[&str] = &["**/tests/**"];
impl Kind {
    pub fn descriptor(self) -> &'static Descriptor {
        match self {
            Self::Structure(kind) => kind.descriptor(),
            Self::ControlFlow(kind) => kind.descriptor(),
            Self::Signature(kind) => kind.descriptor(),
            Self::Ownership(kind) => kind.descriptor(),
            Self::Reference(kind) => kind.descriptor(),
        }
    }
}
