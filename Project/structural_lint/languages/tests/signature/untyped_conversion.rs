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

#[test]
fn rust_flags_into_on_references_unbounded_parameters_and_locals() {
    let source = "fn label<T: Clone, S: Into<String>>(name: &impl Into<String>, count: u32, item: T, text: S) -> Result<Label> {
    let name = name.into();
    let count = count.into();
    let item = item.into();
    let wide = text.try_into()?;
    let local = other.into();
    Ok(Label { name, count, item, wide, local })
}
";
    assert_eq!(
        measured(source, UNTYPED),
        [
            (2, "into".to_string()),
            (3, "into".to_string()),
            (4, "into".to_string()),
            (5, "try_into".to_string()),
            (6, "into".to_string()),
        ]
    );
}

#[test]
fn rust_skips_into_on_parameters_whose_type_bounds_it_by_into() {
    let source = "fn label<P: AsRef<Path> + Into<PathBuf>, R>(name: impl Into<String> + Send, path: P, raw: R) -> Label
where
    R: std::convert::Into<Vec<u8>>,
{
    let name = name.into();
    let path = path.into();
    let raw = raw.into();
    Label { name, path, raw }
}
impl<T: Into<u64>> Counter<T> {
    fn add(&mut self, step: T) {
        let step = step.into();
        self.total += step;
    }
}
trait Scale<U>
where
    U: Into<f64>,
{
    fn scale(&self, factor: U) -> f64 {
        let factor = factor.into();
        factor
    }
}
";
    assert!(measured(source, UNTYPED).is_empty());
}
