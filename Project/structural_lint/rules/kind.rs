use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum Kind {
    NonblankLines,
    LineWidth,
    ForbiddenFile,
    DirectoryEntries,
    NamedIfCondition,
    FunctionLines,
    ParameterCount,
    NestingDepth,
    BooleanParameter,
    BranchBudget,
    UnsafeJustification,
    HardcodedSecret,
    SilentError,
    CommentReference,
    DirectoryArchitecture,
}
impl fmt::Display for Kind {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = serde_json::to_value(self).map_err(|_| fmt::Error)?;
        formatter.write_str(value.as_str().ok_or(fmt::Error)?)
    }
}
