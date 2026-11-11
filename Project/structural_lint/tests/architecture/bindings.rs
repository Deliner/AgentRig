use super::{INCOMPLETE, rust};

fn owned(body: &str) -> String {
    format!(
        "pub struct Value; impl Value {{ fn value() -> i32 {{ crate::b::value() }} \
         fn call() -> i32 {{ {body} }} }} pub fn run() -> i32 {{ Value::call() }}"
    )
}

#[test]
fn self_binding_uses_its_lexical_owner() {
    let behavior = "pub trait Behavior { fn value() -> i32 { crate::b::value() } \
        fn call() -> i32 { Self::value() } } pub fn run() -> i32 { Value::call() } \
        pub struct Value; impl Behavior for Value {}";
    let owners = [
        owned("Self::value()"),
        owned("{ assert_eq!(Self::value(), 7); 7 }"),
        behavior.into(),
    ];
    for source in owners {
        let consumer = rust(&source);
        consumer.compiles("src/lib.rs", &[]);
        consumer.passes();
        consumer.cycles();
    }
    let nested = owned("{ fn nested() -> i32 { Self::value() } nested() }");
    for source in [nested.as_str(), "pub fn run() -> i32 { Self::value() }"] {
        rust(source).fails(INCOMPLETE);
    }
}

#[test]
fn prelude_names_preserve_local_bindings() {
    let standard =
        "pub fn run() -> i32 { let _ = String::new(); let _ = Vec::<u8>::new(); i32::from(7u8) }";
    let consumer = rust(standard);
    consumer.compiles("src/lib.rs", &[]);
    consumer.passes();
    let consumer = rust("use crate::b as String; pub fn run() -> i32 { String::value() }");
    consumer.compiles("src/lib.rs", &[]);
    consumer.passes();
    consumer.cycles();
    for shadowed in [
        "pub fn run<String>() { String::value(); }",
        "pub fn run() { struct String; String::value(); }",
        "use unknown::*; pub fn run() { String::value(); }",
    ] {
        rust(shadowed).fails(INCOMPLETE);
    }
}

#[test]
fn generic_paths_require_one_explicit_bound() {
    let generic = |bound: &str| {
        let consumer = rust(&format!(
            "fn call<{bound}>() -> i32 {{ D::value() }} \
             pub fn run() -> i32 {{ call::<crate::b::Value>() }}"
        ));
        consumer.target(
            "pub trait Access { fn value() -> i32 { 7 } } pub struct Value; impl Access for Value {}",
        );
        consumer
    };
    let bounded = generic("D: crate::b::Access");
    bounded.compiles("src/lib.rs", &[]);
    bounded.passes();
    bounded.cycles();
    for bound in ["D: crate::b::Access + Send", "D"] {
        generic(bound).fails(INCOMPLETE);
    }
}

#[test]
fn local_item_shadows_generic_bound() {
    rust("fn call<D: external::Access>() { struct D; D::value(); }")
        .fails("unresolved Rust name a::D");
}
