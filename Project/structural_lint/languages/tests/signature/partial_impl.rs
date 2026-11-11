use super::measured;
use crate::structural_lint::rules::kind::Kind;
use crate::structural_lint::rules::signature::Signature;

const PARTIAL: Kind = Kind::Signature(Signature::PartialImpl);

#[test]
fn rust_flags_mirror_impls_reading_self_fields_without_naming_every_field() {
    let source = "impl PartialEq for User {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}
impl std::hash::Hash for User {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}
impl Clone for Buffer {
    fn clone(&self) -> Self {
        Self { data: self.data.clone(), ..*self }
    }
}
impl PartialEq for Point {
    fn eq(&self, other: &Self) -> bool {
        let Self { x, .. } = self;
        *x == other.x && self.y == other.y
    }
}
";
    assert_eq!(
        measured(source, PARTIAL),
        [
            (1, "PartialEq for User".to_string()),
            (6, "Hash for User".to_string()),
            (11, "Clone for Buffer".to_string()),
            (16, "PartialEq for Point".to_string()),
        ]
    );
}

#[test]
fn rust_skips_impls_naming_every_field_other_traits_justified_and_test_impls() {
    let source = "impl PartialEq for User {
    fn eq(&self, other: &Self) -> bool {
        let Self { id, name } = self;
        *id == other.id && *name == other.name && self.id > 0
    }
}
impl Clone for Buffer {
    fn clone(&self) -> Self {
        Buffer { data: self.data.clone(), size: self.size }
    }
}
impl Ord for Rank {
    fn cmp(&self, other: &Self) -> Ordering {
        other.cmp(self)
    }
}
impl Display for User {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        write!(formatter, \"{}\", self.name)
    }
}
// DECISION: D001
impl PartialEq for Rank {
    fn eq(&self, other: &Self) -> bool {
        self.score == other.score
    }
}
#[cfg(test)]
mod tests {
    impl PartialEq for Probe {
        fn eq(&self, other: &Self) -> bool {
            self.id == other.id
        }
    }
}
";
    assert!(measured(source, PARTIAL).is_empty());
}
