use super::descriptor::{Descriptor, TEST_CODE};
use super::parameters::Parameters;
use crate::shared::diagnostics::actionable_findings::Level as Severity;
use serde::{Deserialize, Serialize};

// DECISION: D133
#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum Structure {
    NonblankLines,
    LineWidth,
    ForbiddenFile,
    ProgramLanguage,
    DirectoryEntries,
    DumpName,
    FunctionLines,
    EmbeddedText,
    DirectoryArchitecture,
    TestPlacement,
    SameText,
    TestInCodeFile,
}
impl Structure {
    pub(super) fn descriptor(self) -> &'static Descriptor {
        match self {
            Self::NonblankLines => &NONBLANKLINES,
            Self::LineWidth => &LINEWIDTH,
            Self::ForbiddenFile => &FORBIDDENFILE,
            Self::ProgramLanguage => &PROGRAMLANGUAGE,
            Self::DirectoryEntries => &DIRECTORYENTRIES,
            Self::DumpName => &DUMPNAME,
            Self::FunctionLines => &FUNCTIONLINES,
            Self::EmbeddedText => &EMBEDDEDTEXT,
            Self::DirectoryArchitecture => &DIRECTORYARCHITECTURE,
            Self::TestPlacement => &TESTPLACEMENT,
            Self::SameText => &SAMETEXT,
            Self::TestInCodeFile => &TESTINCODEFILE,
        }
    }
}
const NONBLANKLINES: Descriptor = Descriptor {
    default_id: "file-size",
    exclude: &["**/Cargo.lock", "**/package-lock.json", "**/uv.lock"],
    target: "file",
    metric: "nonempty lines, including comments; non-UTF-8 files skipped",
    skill: "refactor-large-file",
    parameters: Parameters::Thresholds {
        warning: Some(300),
        error: 500,
    },
    accepts_exception: false,
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
    accepts_exception: false,
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
    accepts_exception: false,
};
// DECISION: D198
const PROGRAMLANGUAGE: Descriptor = Descriptor {
    default_id: "program-language",
    exclude: &[],
    target: "file",
    metric: "a file whose extension, or for a file without one the interpreter of its #! line, names a known program language outside allowed_languages",
    skill: "rewrite-in-allowed-language",
    parameters: Parameters::Level {
        default: Severity::Error,
        violation: "a program in a language the project does not allow escapes its language rules; rewrite it in an allowed language",
    },
    accepts_exception: false,
};
const DIRECTORYENTRIES: Descriptor = Descriptor {
    default_id: "directory-size",
    exclude: &[],
    target: "directory",
    metric: "immediate child names in the selected inventory, except the architecture.yaml contract",
    skill: "refactor-large-directory",
    parameters: Parameters::Thresholds {
        warning: Some(10),
        error: 15,
    },
    accepts_exception: false,
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
    accepts_exception: false,
};
// DECISION: D189
const FUNCTIONLINES: Descriptor = Descriptor {
    default_id: "function-lines",
    exclude: &[],
    target: "file",
    metric: "nonblank lines from signature through body, including comments; declarations without body excluded",
    skill: "refactor-long-function",
    parameters: Parameters::Thresholds {
        warning: None,
        error: 40,
    },
    accepts_exception: false,
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
    accepts_exception: false,
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
    accepts_exception: false,
};
// DECISION: D145
// DECISION: D223
const TESTPLACEMENT: Descriptor = Descriptor {
    default_id: "test-placement",
    exclude: TEST_CODE,
    target: "file",
    metric: "every Rust file module declared under #[cfg(test)] or inside a test-only module outside a tests directory, at its declaration line, and every Rust tests.rs file or Python test_*.py or *_test.py file outside a tests directory, at line 1",
    skill: "relocate-tests",
    parameters: Parameters::Level {
        default: Severity::Error,
        violation: "test code outside a tests/ directory is measured as production code by every other rule; move it into the tests/ directory of the code it tests",
    },
    accepts_exception: false,
};
// DECISION: D224
const TESTINCODEFILE: Descriptor = Descriptor {
    default_id: "test-in-code-file",
    exclude: TEST_CODE,
    target: "file",
    metric: "every outermost Rust item under #[cfg(test)], including all(test, ..), or under #[test] in a file outside a tests directory, at the item's line, except a module declaration without a body in mod.rs, lib.rs or main.rs",
    skill: "move-test-code-out",
    parameters: Parameters::Level {
        default: Severity::Error,
        violation: "test code in a code file reaches the file's private items; move it into the tests/ directory of this directory, declare it in the entry mod.rs, lib.rs or main.rs and test only what the file opens to its directory",
    },
    accepts_exception: false,
};
// DECISION: D195
const SAMETEXT: Descriptor = Descriptor {
    default_id: "same-text",
    exclude: &[],
    target: "file",
    metric: "every selected file whose bytes differ from the first selected file in path order",
    skill: "condense-markdown",
    parameters: Parameters::Level {
        default: Severity::Error,
        violation: "the files this rule selects must carry one text; merge the intended edits into one text and write it to each of them",
    },
    accepts_exception: false,
};
