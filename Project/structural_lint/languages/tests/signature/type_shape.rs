use super::measured;
use crate::structural_lint::rules::kind::Kind;
use crate::structural_lint::rules::signature::Signature;

const ALIAS: Kind = Kind::Signature(Signature::PrimitiveAlias);
const OPTION: Kind = Kind::Signature(Signature::AmbiguousOption);

#[test]
fn rust_flags_aliases_of_a_scalar_or_string_but_not_generic_or_other_types() {
    let source = "type Meters = f64;
pub type UserId = u64;
type Name = String;
type Flag = bool;
type Pair<T> = (T, T);
type Wide<T> = u64;
type Store = HashMap<String, u64>;
type Text = std::string::String;
// DECISION: D001
type Seconds = u32;
";
    assert_eq!(
        measured(source, ALIAS),
        [
            (1, "Meters".to_string()),
            (2, "UserId".to_string()),
            (3, "Name".to_string()),
        ]
    );
}

// DECISION: D162
#[test]
fn rust_flags_an_option_over_a_bool_wherever_it_stands_and_leaves_a_nested_option_to_clippy() {
    let source = "struct Settings {
    verbose: Option<bool>,
    timeout: Option<Option<u32>>,
    limit: Option<u32>,
}
fn choose(flag: std::option::Option<bool>) -> Option<Vec<bool>> {
    None
}
// DECISION: D001
type Tristate = Option<bool>;
";
    assert_eq!(
        measured(source, OPTION),
        [
            (2, "Option<bool>".to_string()),
            (6, "std::option::Option<bool>".to_string()),
        ]
    );
}

const NESTED_OPTION_PLACES: &str = r#"use std::collections::HashMap;
pub struct Fields {
    pub plain: Option<Option<u32>>,
    pub wrapped: Vec<Option<Option<u32>>>,
    pub mapped: HashMap<u32, Option<Option<u32>>>,
    pub pointer: *const Option<Option<u32>>,
    pub qualified: <Option<Option<u32>> as Shape>::Out,
    pub triple: Option<Option<Option<u32>>>,
    pub callback: fn(Option<Option<u32>>),
    pub produce: fn() -> Option<Option<u32>>,
    pub boxed: Box<dyn Fn(Option<Option<u32>>)>,
    pub items: Box<dyn Iterator<Item = Option<Option<u32>>>>,
}
pub struct Pair(pub Option<Option<u32>>);
pub enum Event {
    Changed { value: Option<Option<u32>> },
}
pub trait Shape {
    type Out;
    const LIMIT: Option<Option<u32>>;
    fn read(&self) -> Option<Option<u32>>;
    fn fallback(value: Option<Option<u32>>) -> u32 {
        value.flatten().unwrap_or(0)
    }
}
impl Shape for Option<Option<u32>> {
    type Out = Option<Option<u32>>;
    const LIMIT: Option<Option<u32>> = None;
    fn read(&self) -> Option<Option<u32>> {
        const INNER: Option<Option<u32>> = None;
        INNER
    }
}
impl From<Option<Option<u32>>> for Pair {
    fn from(value: Option<Option<u32>>) -> Self {
        Self(value)
    }
}
impl Pair {
    pub const EMPTY: Option<Option<u32>> = None;
    pub fn take(&self, value: Option<Option<u32>>) -> u32 {
        let local: Option<Option<u32>> = value;
        let pick = |item: Option<Option<u32>>| item.flatten();
        let made = || -> Option<Option<u32>> { None };
        let _size = std::mem::size_of::<Option<Option<u32>>>();
        pick(local.or(made())).unwrap_or(0)
    }
}
pub fn bounded<T: From<Option<Option<u32>>>>(value: T) -> T {
    value
}
pub fn constrained<T>(value: T) -> T
where
    T: From<Option<Option<u32>>>,
{
    value
}
pub struct Defaulted<T = Option<Option<u32>>>(pub T);
pub fn opaque() -> impl Iterator<Item = Option<Option<u32>>> {
    std::iter::empty()
}
pub type Alias = Option<Option<u32>>;
pub static GLOBAL: Option<Option<u32>> = None;
pub const CONSTANT: Option<Option<u32>> = None;
unsafe extern "C" {
    pub fn foreign(value: *const Option<Option<u32>>);
    pub static FOREIGN: *const Option<Option<u32>>;
}
pub mod inline {
    pub fn inner(value: Option<Option<u32>>) -> Option<u32> {
        value.flatten()
    }
}
pub type Deep = Option<Option<Option<u32>>>;
// DECISION: D001
pub type Justified = Option<Option<u32>>;
"#;

// DECISION: D163
// DECISION: D164
#[test]
fn rust_flags_a_nested_option_once_where_neither_clippy_nor_the_trait_reports_it() {
    let nested = "Option<Option<u32>>".to_string();
    let unchecked = [
        9, 10, 11, 12, 26, 27, 34, 42, 43, 44, 45, 49, 54, 58, 59, 62, 66, 67,
    ];
    let mut expected: Vec<_> = unchecked.map(|line| (line, nested.clone())).into();
    expected.push((74, "Option<Option<Option<u32>>>".to_string()));
    assert_eq!(measured(NESTED_OPTION_PLACES, OPTION), expected);
}

// DECISION: D164
#[test]
fn rust_leaves_a_nested_option_in_a_trait_impl_member_signature_to_the_trait() {
    let source = "pub trait Visit {
    const PICK: fn(Option<Option<u32>>);
    fn visit(&self, each: fn(Option<Option<u32>>));
    fn items(&self) -> Box<dyn Iterator<Item = Option<Option<u32>>>>;
    fn bounded<T: From<Option<Option<u32>>>>(value: T) -> T;
}
impl Visit for Pair {
    const PICK: fn(Option<Option<u32>>) = drop;
    fn visit(&self, each: fn(Option<Option<u32>>)) {
        let seen: Option<Option<u32>> = None;
        let pick = |item: Option<Option<u32>>| item;
        let _size = std::mem::size_of::<Option<Option<u32>>>();
        each(pick(seen))
    }
    fn items(&self) -> Box<dyn Iterator<Item = Option<Option<u32>>>> {
        Box::new(std::iter::empty())
    }
    fn bounded<T: From<Option<Option<u32>>>>(value: T) -> T {
        value
    }
}
impl Pair {
    pub fn visit(&self, each: fn(Option<Option<u32>>)) {
        each(None)
    }
}
";
    let flagged = [2, 3, 4, 5, 10, 11, 12, 18, 23];
    let expected: Vec<_> = flagged
        .map(|line| (line, "Option<Option<u32>>".to_string()))
        .into();
    assert_eq!(measured(source, OPTION), expected);
}
