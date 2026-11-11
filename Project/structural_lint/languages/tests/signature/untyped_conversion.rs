use super::measured;
use crate::structural_lint::rules::kind::Kind;
use crate::structural_lint::rules::signature::Signature;

const UNTYPED: Kind = Kind::Signature(Signature::UntypedConversion);

#[test]
fn rust_flags_untyped_lets_over_a_generic_conversion() {
    let source = "fn run(text: &str, items: &[u32]) -> Result<(), Error> {
    let port = text.parse()?;
    let total = items.iter().sum();
    let names = items.iter().map(name).collect();
    let wide = small.into();
    let narrow = wide.try_into()?;
    let product = items.iter().product();
    let config = Default::default();
    let error = From::from(cause);
    Ok(())
}
";
    assert_eq!(
        measured(source, UNTYPED),
        [
            (2, "parse".to_string()),
            (3, "sum".to_string()),
            (4, "collect".to_string()),
            (5, "into".to_string()),
            (6, "try_into".to_string()),
            (7, "product".to_string()),
            (8, "default".to_string()),
            (9, "from".to_string()),
        ]
    );
}

#[test]
fn rust_skips_typed_turbofish_argument_pattern_justified_and_test_lets() {
    let source = "fn run(text: &str, items: &[u32]) -> Result<(), Error> {
    let port: u16 = text.parse()?;
    let total = items.iter().sum::<u32>();
    let names = items.iter().collect::<Vec<_>>();
    let parsed = parser.parse(text);
    let (left, right) = pair.into();
    let config = Config::default();
    let error = Error::from(cause);
    // DECISION: D001
    let wide = small.into();
    Ok(())
}
#[cfg(test)]
mod tests {
    fn check(text: &str) {
        let port = text.parse()?;
    }
}
";
    assert!(measured(source, UNTYPED).is_empty());
}
