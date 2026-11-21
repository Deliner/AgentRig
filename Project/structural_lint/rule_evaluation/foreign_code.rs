use crate::structural_lint::languages::Project;
use crate::structural_lint::rule_identities::Settings;

// DECISION: D229
pub(super) struct Declared<'a> {
    pub(super) settings: &'a Settings,
}
impl Project for Declared<'_> {
    fn foreign_crate(&self, name: &str) -> bool {
        self.settings.external.rust.contains(name)
    }
}
