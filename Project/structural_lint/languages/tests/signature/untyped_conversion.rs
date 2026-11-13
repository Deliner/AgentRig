use super::measured;
use crate::structural_lint::rule_identities::kind::Kind;
use crate::structural_lint::rule_identities::signature::Signature;

const UNTYPED: Kind = Kind::Signature(Signature::UntypedConversion);
const TRAIT_PATH: Kind = Kind::Signature(Signature::TraitPathCall);

// DECISION: D171
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

#[test]
fn rust_flags_conversion_trait_paths_whose_target_type_the_statement_does_not_name() {
    let source = "fn convert(bytes: Vec<u8>, text: &str, values: &[u8]) -> Result<Wrapper, Error> {
    take(Into::into(bytes));
    store(std::convert::TryFrom::try_from(text)?);
    let parsed = FromStr::from_str(text)?;
    values.iter().copied().map(From::from).for_each(store);
    let sum = 1 + core::convert::TryInto::try_into(2)?;
    Ok(Wrapper::from(parsed))
}
";
    assert_eq!(
        measured(source, TRAIT_PATH),
        [
            (2, "Into::into".to_string()),
            (3, "TryFrom::try_from".to_string()),
            (4, "FromStr::from_str".to_string()),
            (5, "From::from".to_string()),
            (6, "TryInto::try_into".to_string()),
        ]
    );
}

#[test]
fn rust_skips_typed_owned_turbofish_other_owner_foreign_justified_and_test_trait_paths() {
    let source = "fn convert(bytes: Vec<u8>, text: &str) -> Result<u64, Error> {
    let wide: u64 = From::from(1u8) + u64::from(2u8);
    let owned = From::from(text);
    let value: Wrapper = Into::into(bytes);
    let typed = From::<u8>::from(1);
    let other = Wrapper::from(1);
    let foreign = mine::convert::From::from(1);
    // DECISION: D001
    take(Into::into(bytes));
    Ok(wide)
}
#[cfg(test)]
mod tests {
    fn check(bytes: Vec<u8>) {
        take(Into::into(bytes));
    }
}
";
    assert!(measured(source, TRAIT_PATH).is_empty());
}
