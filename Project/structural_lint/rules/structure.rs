use super::descriptor::{Descriptor, TEST_CODE};
use super::parameters::Parameters;
use crate::shared::diagnostics::lint::Level as Severity;
use serde::Serialize;

// DECISION: D133
#[derive(Clone, Copy, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum Structure {
    NonblankLines,
    LineWidth,
    ForbiddenFile,
    DirectoryEntries,
    DumpName,
    FunctionLines,
    EmbeddedText,
    DirectoryArchitecture,
}
impl Structure {
    pub(super) fn descriptor(self) -> &'static Descriptor {
        match self {
            Self::NonblankLines => &NONBLANKLINES,
            Self::LineWidth => &LINEWIDTH,
            Self::ForbiddenFile => &FORBIDDENFILE,
            Self::DirectoryEntries => &DIRECTORYENTRIES,
            Self::DumpName => &DUMPNAME,
            Self::FunctionLines => &FUNCTIONLINES,
            Self::EmbeddedText => &EMBEDDEDTEXT,
            Self::DirectoryArchitecture => &DIRECTORYARCHITECTURE,
        }
    }
}
const NONBLANKLINES: Descriptor = Descriptor {
    default_id: "file-size",
    exclude: &[],
    target: "file",
    metric: "nonempty lines, including comments; non-UTF-8 files skipped",
    skill: "refactor-large-file",
    parameters: Parameters::Thresholds {
        warning: Some(300),
        error: 500,
    },
};
// DECISION: D036
const LINEWIDTH: Descriptor = Descriptor {
    default_id: "line-width",
    exclude: &[],
    target: "file",
    metric: "characters per line, one finding per line over a threshold; non-UTF-8 files skipped",
    skill: "condense-markdown",
    parameters: Parameters::Thresholds {
        warning: None,
        error: 120,
    },
};
const FORBIDDENFILE: Descriptor = Descriptor {
    default_id: "forbidden-file",
    exclude: &[],
    target: "file",
    metric: "every selected file is a finding, regardless of content",
    skill: "condense-markdown",
    parameters: Parameters::Level {
        default: Severity::Error,
        violation: "file is not allowed here; move needed content to its owner and delete the file",
    },
};
const DIRECTORYENTRIES: Descriptor = Descriptor {
    default_id: "directory-size",
    exclude: &[],
    target: "directory",
    metric: "immediate child names in the selected inventory",
    skill: "refactor-large-directory",
    parameters: Parameters::Thresholds {
        warning: Some(10),
        error: 15,
    },
};
// DECISION: D072
const DUMPNAME: Descriptor = Descriptor {
    default_id: "dump-name",
    exclude: &[],
    target: "directory",
    metric: "a directory whose name is a generic dumping ground: utils, helpers, misc or common; shared is allowed",
    skill: "refactor-large-directory",
    parameters: Parameters::Level {
        default: Severity::Error,
        violation: "a directory named for a generic dumping ground hides mixed responsibilities; name it for what it owns or distribute its contents to cohesive owners; shared is for genuine cross-feature leaves",
    },
};
const FUNCTIONLINES: Descriptor = Descriptor {
    default_id: "function-size",
    exclude: &[],
    target: "file",
    metric: "nonblank lines from signature through body, including comments; declarations without body excluded",
    skill: "refactor-long-function",
    parameters: Parameters::Thresholds {
        warning: None,
        error: 40,
    },
};
// DECISION: D122
const EMBEDDEDTEXT: Descriptor = Descriptor {
    default_id: "embedded-text",
    exclude: TEST_CODE,
    target: "file",
    metric: "lines spanned by one Rust string literal, plain, raw or byte, including one inside macro arguments, outside test code",
    skill: "extract-embedded-text",
    parameters: Parameters::Thresholds {
        warning: None,
        error: 20,
    },
};
const DIRECTORYARCHITECTURE: Descriptor = Descriptor {
    default_id: "architecture",
    exclude: &[],
    target: "directory",
    metric: "architecture.yaml inventories and directory cycles in resolved source dependencies; incomplete analysis is a finding",
    skill: "refactor-large-directory",
    parameters: Parameters::Level {
        default: Severity::Error,
        violation: "architecture contract is violated",
    },
};
