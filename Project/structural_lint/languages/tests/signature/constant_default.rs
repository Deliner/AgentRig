use super::measured;
use crate::structural_lint::rules::kind::Kind;
use crate::structural_lint::rules::signature::Signature;

const CONSTANT: Kind = Kind::Signature(Signature::ConstantDefault);

#[test]
fn rust_flags_trait_default_methods_whose_body_ignores_self() {
    let source = "trait Plugin {
    fn name(&self) -> &str {
        \"plugin\"
    }
    fn enabled() -> bool {
        true
    }
    fn label(&self) -> String {
        format!(\"{}!\", self.name())
    }
    fn hook(&self) {}
    fn run(&self);
    // DECISION: D001
    fn priority(&self) -> u8 {
        0
    }
}
#[cfg(test)]
mod tests {
    trait Probe {
        fn size(&self) -> usize {
            1
        }
    }
}
";
    assert_eq!(
        measured(source, CONSTANT),
        [(2, "name".to_string()), (5, "enabled".to_string())]
    );
}
