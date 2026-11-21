use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum CheckKind {
    Command,
    Lint,
    Memory,
    Tools,
}
#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Check {
    pub id: String,
    pub kind: CheckKind,
    pub command: Option<String>,
    #[serde(default = "all")]
    pub include: Vec<String>,
    pub skill: String,
    #[serde(default)]
    pub warning: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub affected: Vec<AffectedTests>,
}
#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AffectedTests {
    pub include: Vec<String>,
    pub targets: Vec<String>,
}
fn all() -> Vec<String> {
    vec!["**".into()]
}
