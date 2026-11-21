use super::measured_within;
use crate::structural_lint::languages::{Imported, Project, PythonImport};
use crate::structural_lint::rule_identities::{ControlFlow, Kind, Signature, Structure};
use std::path::Path;

const COUNT: Kind = Kind::Signature(Signature::ParameterCount);
const FLAG: Kind = Kind::Signature(Signature::BooleanParameter);
const TUPLE: Kind = Kind::Signature(Signature::TupleAsRecord);

struct WithRustls;
impl Project for WithRustls {
    fn foreign_crate(&self, name: &str) -> bool {
        ["std", "core", "alloc", "rustls"].contains(&name)
    }
    fn python_import(&self, _from: &Path, _import: PythonImport<'_>) -> Imported {
        Imported::Unknown
    }
}

// DECISION: D229
fn measured(source: &str, kinds: &[Kind]) -> Vec<(usize, String)> {
    measured_within(Path::new("source.rs"), source, &WithRustls, kinds)
}

#[test]
fn rust_leaves_the_methods_of_traits_imported_from_a_declared_external_crate() {
    let source = "use rustls::client::danger::{self, ServerCertVerified, ServerCertVerifier};
use rustls::client::danger::ServerCertVerifier as Verifying;
impl ServerCertVerifier for Strict {
    fn verify_server_cert(&self, entity: &Der, chain: &[Der], name: &Name, ocsp: &[u8], now: Time) -> Verdict {
        Ok(ServerCertVerified::assertion())
    }
}
impl Verifying for Aliased {
    fn verify(&self, strict: bool) {}
}
impl danger::ServerCertVerifier for Listed {
    fn verify(&self, strict: bool) {}
}
impl ::rustls::Signer for Global {
    fn sign(&self, strict: bool) {}
}
fn build() {
    use rustls::Signer as Local;
    impl Local for Nested {
        fn sign(&self, strict: bool) {}
    }
}
";
    let found = measured(source, &[COUNT, FLAG, TUPLE]);
    assert_eq!(found, [(17, "build".to_string())]);
}

#[test]
fn rust_leaves_the_methods_of_standard_traits_and_the_functions_of_extern_blocks() {
    let source = "use std::fmt;
impl Iterator for Cells {
    type Item = u8;
    fn next(&mut self) -> Option<u8> {
        None
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, None)
    }
}
impl fmt::Display for Cell {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        Ok(())
    }
}
impl From<(u8, u8)> for Cell {
    fn from(pair: (u8, u8)) -> Self {
        Cell
    }
}
extern \"C\" {
    fn compress(source: *const u8, length: usize, target: *mut u8, capacity: usize, level: i32) -> i32;
    fn reset(strict: bool) -> (i32, i32);
}
";
    assert!(measured(source, &[COUNT, FLAG, TUPLE]).is_empty());
}

#[test]
fn rust_judges_the_methods_of_traits_the_project_declares() {
    let source = "use crate::store::Store;
trait Sink {
    fn write(&self, a: u8, b: u8, c: u8, d: u8, e: u8);
}
impl Sink for File {
    fn write(&self, a: u8, b: u8, c: u8, d: u8, e: u8) {}
}
impl Store for File {
    fn put(&self, strict: bool) {}
}
impl super::Codec for File {
    fn decode(&self) -> (u8, u8) {
        (0, 0)
    }
}
mod rustls {
    pub trait Signer {}
}
impl rustls::Signer for File {
    fn sign(&self, strict: bool) {}
}
";
    let counted: Vec<_> = measured(source, &[COUNT])
        .into_iter()
        .filter(|(_, symbol)| symbol == "write")
        .collect();
    assert_eq!(
        counted,
        [(3, "write".to_string()), (6, "write".to_string())]
    );
    assert_eq!(
        measured(source, &[FLAG]),
        [
            (9, "strict: bool".to_string()),
            (20, "strict: bool".to_string())
        ]
    );
    assert_eq!(measured(source, &[TUPLE]), [(12, "signature".to_string())]);
}

#[test]
fn rust_judges_the_methods_of_traits_it_cannot_place() {
    let source = "use rustls::client::*;
impl ServerCertVerifier for Globbed {
    fn verify(&self, strict: bool) {}
}
impl Display for Unresolved {
    fn fmt(&self, strict: bool) {}
}
";
    assert_eq!(
        measured(source, &[FLAG]),
        [
            (3, "strict: bool".to_string()),
            (6, "strict: bool".to_string())
        ]
    );
}

#[test]
fn rust_measures_the_bodies_of_foreign_trait_methods() {
    let source = "impl Iterator for Cells {
    type Item = u8;
    fn next(&mut self) -> Option<u8> {
        if self.open {
            if self.ready {
                return Some(0);
            }
        }
        None
    }
}
";
    let body = [
        Kind::Structure(Structure::FunctionLines),
        Kind::ControlFlow(ControlFlow::NestingDepth),
    ];
    assert_eq!(
        measured(source, &body),
        [(3, "next".to_string()), (3, "next".to_string())]
    );
}
