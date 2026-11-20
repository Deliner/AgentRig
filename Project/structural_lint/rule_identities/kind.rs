use super::control_flow::ControlFlow;
use super::descriptor::Descriptor;
use super::ownership::Ownership;
use super::reference::Reference;
use super::signature::Signature;
use super::structure::Structure;
use serde::Serialize;
use std::fmt;

// DECISION: D133
#[derive(Clone, Copy, Debug, Serialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum Kind {
    Structure(Structure),
    ControlFlow(ControlFlow),
    Signature(Signature),
    Ownership(Ownership),
    Reference(Reference),
}
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
impl fmt::Display for Kind {
    // DECISION: D129 allow-justification
    #[expect(clippy::map_err_ignore)]
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = serde_json::to_value(self).map_err(|_| fmt::Error)?;
        formatter.write_str(value.as_str().ok_or(fmt::Error)?)
    }
}
