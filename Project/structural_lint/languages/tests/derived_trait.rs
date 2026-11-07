use crate::structural_lint::languages::analyze;
use crate::structural_lint::rules::kind::Kind;
use crate::structural_lint::rules::signature::Signature;
use std::path::Path;

const SECRET_DEBUG: Kind = Kind::Signature(Signature::SecretDebug);
const UNCHECKED: Kind = Kind::Signature(Signature::UncheckedDeserialize);

fn measured(source: &str, kind: Kind) -> Vec<(usize, String)> {
    let analysis = analyze(Path::new("source.rs"), source).unwrap();
    assert_eq!(analysis.parse_error, None);
    let mut found: Vec<_> = analysis
        .measurements
        .into_iter()
        .filter(|item| item.kind == kind)
        .map(|item| (item.line, item.symbol))
        .collect();
    found.sort();
    found
}

// DECISION: D130
#[test]
fn rust_flags_secret_named_string_and_byte_fields_of_a_derived_debug_struct() {
    let source = "#[derive(Clone, Debug)]
struct Login {
    user: String,
    password: String,
    token: Option<String>,
    api_key: std::string::String,
}
/// Settings read from the environment.
#[cfg_attr(feature = \"trace\", derive(Debug))]
struct Remote<'a> {
    secret: &'a str,
    private_key: Vec<u8>,
    jwt: Option<&'static str>,
}
";
    assert_eq!(
        measured(source, SECRET_DEBUG),
        vec![
            (4, "Login.password".to_string()),
            (5, "Login.token".to_string()),
            (6, "Login.api_key".to_string()),
            (11, "Remote.secret".to_string()),
            (12, "Remote.private_key".to_string()),
            (13, "Remote.jwt".to_string()),
        ]
    );
}

#[test]
fn rust_skips_manual_debug_masked_types_other_names_tuple_structs_and_test_code() {
    let source = "struct Manual {
    password: String,
}
impl fmt::Debug for Manual {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(\"Manual\")
    }
}
#[derive(Debug)]
struct Masked {
    password: Redacted<String>,
    secret: Option<Redacted<String>>,
    token: Vec<u32>,
    db_password: String,
    cvv: u16,
}
#[derive(Debug)]
struct Pair(String, String);
#[cfg(test)]
mod tests {
    #[derive(Debug)]
    struct Fixture {
        password: String,
    }
}
";
    assert_eq!(measured(source, SECRET_DEBUG), Vec::new());
}

// DECISION: D131
#[test]
fn rust_flags_a_derived_deserialize_beside_an_inherent_constructor_refusing_a_stored_parameter() {
    let source = "#[derive(Debug, Deserialize)]
pub struct Port {
    number: u16,
}
impl Port {
    pub fn new(number: u16) -> Result<Self, Error> {
        if number == 0 {
            return Err(Error::Zero);
        }
        Ok(Self { number })
    }
}
/// Closed range read from settings.
#[cfg_attr(feature = \"serde\", derive(serde::Deserialize))]
struct Interval<T> {
    start: T,
    end: T,
}
impl<T: PartialOrd> Interval<T> {
    fn new(start: T, end: T) -> Option<Self> {
        (start <= end).then_some(Self { start, end })
    }
}
";
    let port = (2, "Port::new".to_string());
    let interval = (15, "Interval::new".to_string());
    assert_eq!(measured(source, UNCHECKED), [port, interval]);
}

#[test]
fn rust_flags_try_from_and_from_str_constructors_of_tuple_and_named_structs() {
    let source = "#[derive(Deserialize)]
struct Limit(u32);
impl TryFrom<u32> for Limit {
    type Error = Error;
    fn try_from(value: u32) -> Result<Self, Self::Error> {
        if value > 100 { Err(Error::Large) } else { Ok(Self(value)) }
    }
}
#[derive(Deserialize)]
struct Label {
    text: String,
}
impl std::str::FromStr for Label {
    type Err = Error;
    fn from_str(text: &str) -> Result<Self, Self::Err> {
        ensure!(!text.is_empty(), \"empty label\");
        Ok(Label { text: text.to_owned() })
    }
}
";
    let limit = (2, "Limit::try_from".to_string());
    let label = (10, "Label::from_str".to_string());
    assert_eq!(measured(source, UNCHECKED), [limit, label]);
}

#[test]
fn rust_flags_a_bail_refusal_through_a_result_alias_and_a_converted_mutable_parameter() {
    let source = "#[derive(Deserialize)]
struct Ratio {
    value: f64,
}
impl Ratio {
    fn parse(mut value: f64) -> anyhow::Result<Ratio> {
        if value.is_nan() {
            bail!(\"not a number\");
        }
        value = value.abs();
        Ok(Self { value: value.into() })
    }
}
";
    let ratio = (2, "Ratio::parse".to_string());
    assert_eq!(measured(source, UNCHECKED), [ratio]);
}

#[test]
fn rust_skips_deserialization_routed_through_serde_conversion_or_field_attributes() {
    let source = "#[derive(Deserialize)]
#[serde(try_from = \"RawRange\")]
struct Range {
    low: u8,
}
impl Range {
    fn new(low: u8) -> Result<Self, Error> {
        if low > 9 { return Err(Error::Large); }
        Ok(Self { low })
    }
}
#[derive(Deserialize)]
#[serde(from = \"RawSpan\")]
struct Span(u8);
impl Span {
    fn new(width: u8) -> Option<Self> { (width > 0).then_some(Self(width)) }
}
#[derive(Deserialize)]
struct Bounded {
    #[serde(deserialize_with = \"positive\")]
    count: i32,
    name: String,
}
impl Bounded {
    fn new(count: i32, name: String) -> Result<Self, Error> {
        if count < 0 { return Err(Error::Negative); }
        Ok(Self { count, name })
    }
}
#[derive(Deserialize)]
struct Wrapped(#[serde(with = \"checked\")] pub u8);
impl Wrapped {
    fn new(level: u8) -> Option<Self> { if level > 3 { None } else { Some(Self(level)) } }
}
";
    assert_eq!(measured(source, UNCHECKED), Vec::new());
}

#[test]
fn rust_skips_a_manual_deserialize_and_unchecked_unstored_or_receiver_parameters() {
    let source = "#[derive(Deserialize)]
struct Manual {
    size: usize,
}
impl Manual {
    fn new(size: usize) -> Result<Self, Error> {
        ensure!(size > 0, \"empty\");
        Ok(Self { size })
    }
}
impl<'de> Deserialize<'de> for Manual {
    fn deserialize<D: Deserializer<'de>>(input: D) -> Result<Self, D::Error> {
        Self::new(usize::deserialize(input)?).map_err(D::Error::custom)
    }
}
#[derive(Deserialize)]
struct Unchecked {
    size: usize,
    limit: usize,
}
impl Unchecked {
    fn new(size: usize, limit: usize) -> Result<Self, Error> {
        if limit == 0 { return Err(Error::Zero); }
        Ok(Self { size, limit: 10 })
    }
    fn resize(&self, size: usize) -> Option<Self> {
        (size > 0).then_some(Self { size, limit: self.limit })
    }
}
";
    assert_eq!(measured(source, UNCHECKED), Vec::new());
}

#[test]
fn rust_skips_other_trait_constructors_structs_without_the_derive_and_test_code() {
    let source = "#[derive(Deserialize)]
struct Depth(u8);
impl Validate for Depth {
    fn build(depth: u8) -> Result<Self, Error> {
        if depth > 9 { return Err(Error::Deep); }
        Ok(Self(depth))
    }
}
struct Plain {
    size: usize,
}
impl Plain {
    fn new(size: usize) -> Option<Self> { (size > 0).then_some(Self { size }) }
}
#[cfg(test)]
mod tests {
    #[derive(Deserialize)]
    struct Fixture {
        size: usize,
    }
    impl Fixture {
        fn new(size: usize) -> Option<Self> { (size > 0).then_some(Self { size }) }
    }
}
";
    assert_eq!(measured(source, UNCHECKED), Vec::new());
}
