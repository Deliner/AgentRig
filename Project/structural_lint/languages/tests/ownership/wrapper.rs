use super::measured;
use crate::structural_lint::rules::ownership::Ownership;

#[test]
fn rust_flags_one_visible_field_per_struct_whose_constructor_checks_it() {
    let source = "pub struct Port {
    pub number: u16,
    pub label: String,
}
impl Port {
    pub fn new(number: u16, label: String) -> Result<Self, Error> {
        if number == 0 || label.is_empty() {
            return Err(Error::Invalid);
        }
        Ok(Self { number, label })
    }
    pub fn parse(text: &str) -> Option<Self> {
        let number = text.parse().ok()?;
        (number > 0).then_some(Self { number, label: text.into() })
    }
}
pub struct Limit(pub(crate) u32);
impl TryFrom<u32> for Limit {
    type Error = Error;
    fn try_from(value: u32) -> Result<Self, Self::Error> {
        if value > 100 { Err(Error::Large) } else { Ok(Self(value)) }
    }
}
";
    assert_eq!(
        measured(source, &[], Ownership::OpenCheckedField),
        [
            (1, "Port::new".to_string()),
            (17, "Limit::try_from".to_string())
        ]
    );
}

#[test]
fn rust_skips_private_fields_unchecked_constructors_and_justified_structs() {
    let source = "pub struct Port {
    number: u16,
}
impl Port {
    pub fn new(number: u16) -> Option<Self> {
        (number > 0).then_some(Self { number })
    }
}
pub struct Point {
    pub x: i32,
}
impl Point {
    pub fn new(x: i32) -> Self {
        Self { x }
    }
}
// DECISION: D001
#[repr(C)]
pub struct Header {
    pub size: u32,
}
impl Header {
    pub fn new(size: u32) -> Option<Self> {
        (size > 0).then_some(Self { size })
    }
}
";
    assert!(measured(source, &[], Ownership::OpenCheckedField).is_empty());
}

#[test]
fn rust_flags_deref_to_a_concrete_inner_type_but_not_to_a_type_parameter() {
    let source = "struct Name(String);
impl Deref for Name {
    type Target = String;
    fn deref(&self) -> &String {
        &self.0
    }
}
impl DerefMut for Name {
    fn deref_mut(&mut self) -> &mut String {
        &mut self.0
    }
}
struct Guard<T>(T);
impl<T> std::ops::Deref for Guard<T> {
    type Target = T;
    fn deref(&self) -> &T {
        &self.0
    }
}
impl Deref for External {
    type Target = u8;
    fn deref(&self) -> &u8 {
        &0
    }
}
struct Bytes(Vec<u8>);
// DECISION: D001
impl Deref for Bytes {
    type Target = [u8];
    fn deref(&self) -> &[u8] {
        &self.0
    }
}
";
    assert_eq!(
        measured(source, &[], Ownership::WrapperDeref),
        [(2, "Name".to_string()), (8, "Name".to_string())]
    );
}

#[test]
fn rust_flags_from_a_raw_value_into_a_one_field_struct() {
    let source = "struct Id(u64);
impl From<u64> for Id {
    fn from(value: u64) -> Self { Self(value) }
}
struct Name {
    text: String,
}
impl From<&str> for Name {
    fn from(text: &str) -> Self { Self { text: text.into() } }
}
impl From<String> for Name {
    fn from(text: String) -> Self { Self { text } }
}
struct Span(usize, usize);
impl From<usize> for Span {
    fn from(start: usize) -> Self { Self(start, start) }
}
impl From<Uuid> for Id {
    fn from(value: Uuid) -> Self { Self(value.as_u64()) }
}
// DECISION: D001
impl From<u32> for Id {
    fn from(value: u32) -> Self { Self(value.into()) }
}
";
    assert_eq!(
        measured(source, &[], Ownership::PrimitiveFrom),
        [
            (2, "Id".to_string()),
            (8, "Name".to_string()),
            (11, "Name".to_string()),
        ]
    );
}
