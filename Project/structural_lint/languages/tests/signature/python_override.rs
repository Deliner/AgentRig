use super::measured_within;
use crate::structural_lint::languages::{Imported, Project, PythonImport, Source};
use crate::structural_lint::rule_identities::{ControlFlow, Kind, Signature, Structure};
use std::path::{Path, PathBuf};

const COUNT: Kind = Kind::Signature(Signature::ParameterCount);
const FLAG: Kind = Kind::Signature(Signature::BooleanParameter);
const BASE: &str = "import logging

class Channel(logging.Handler):
    pass

class Framed(logging.Handler):
    def emit(self, record):
        pass
";

struct Packages;
impl Project for Packages {
    fn foreign_crate(&self, _name: &str) -> bool {
        false
    }
    fn python_import(&self, _from: &Path, import: PythonImport<'_>) -> Imported {
        match import.module {
            "logging" | "typing" => Imported::Outside,
            "app.base" => Imported::Inside(vec![Source {
                path: PathBuf::from("app/base.py"),
                text: BASE.to_owned(),
            }]),
            _ => Imported::Unknown,
        }
    }
}

// DECISION: D229
fn measured(source: &str, kinds: &[Kind]) -> Vec<(usize, String)> {
    measured_within(Path::new("source.py"), source, &Packages, kinds)
}
fn flagged(lines: &[usize]) -> Vec<(usize, String)> {
    lines
        .iter()
        .map(|line| (*line, "strict: bool".to_string()))
        .collect()
}

#[test]
fn python_leaves_typing_overrides_of_classes_outside_the_project() {
    let source = "import dataclasses
import logging
import typing
from logging import Handler
from typing import override

class Recorder(logging.Logger):
    @override
    def makeRecord(self, name, level, fn, lno, msg, args, exc_info, func=None, extra=None, sinfo=None):
        pass

@dataclasses.dataclass
class Console(Handler):
    @typing.override
    def handleError(self, record, strict: bool = False):
        pass

class Plain:
    @override
    def __format__(self, spec, strict: bool = False):
        pass
";
    assert!(measured(source, &[COUNT, FLAG]).is_empty());
}

#[test]
fn python_follows_project_bases_and_judges_names_a_project_class_declares() {
    let source = "from typing import override
from app.base import Channel, Framed

class Local(Channel):
    pass

class Through(Local):
    @override
    def emit(self, record, strict: bool):
        pass

class Declared(Framed):
    @override
    def emit(self, record, strict: bool):
        pass

class Own(Local):
    def flush(self, strict: bool):
        pass

class Child(Own):
    @override
    def flush(self, strict: bool):
        pass
";
    assert_eq!(measured(source, &[FLAG]), flagged(&[14, 18, 23]));
}

#[test]
fn python_judges_overrides_whose_bases_it_cannot_trace() {
    let source = "import logging
from typing import override
from unknown.place import Base
from vendor import *

class Starred(Mixin):
    @override
    def emit(self, record, strict: bool):
        pass

class Unknown(Base):
    @override
    def emit(self, record, strict: bool):
        pass

class Mixed(logging.Handler, Base):
    @override
    def emit(self, record, strict: bool):
        pass

class Called(make_base()):
    @override
    def emit(self, record, strict: bool):
        pass
";
    assert_eq!(measured(source, &[FLAG]), flagged(&[8, 13, 18, 23]));
}

#[test]
fn python_judges_methods_without_override_and_structural_protocol_members() {
    let source = "import logging
from typing import Protocol

class Sink(Protocol):
    def write(self, data, strict: bool): ...

class File:
    def write(self, data, strict: bool):
        pass

class Recorder(logging.Logger):
    def makeRecord(self, name, level, fn, lno, msg, args, exc_info, func=None, extra=None, sinfo=None):
        pass
";
    assert_eq!(measured(source, &[FLAG]), flagged(&[5, 8]));
    let counted: Vec<_> = measured(source, &[COUNT])
        .into_iter()
        .map(|(line, _)| line)
        .collect();
    assert_eq!(counted, [5, 8, 12]);
}

#[test]
fn python_measures_the_bodies_of_foreign_overrides() {
    let source = "import logging
from typing import override

class Recorder(logging.Logger):
    @override
    def handle(self, record):
        if record:
            if self.disabled:
                return
";
    let body = [
        Kind::Structure(Structure::FunctionLines),
        Kind::ControlFlow(ControlFlow::NestingDepth),
    ];
    assert_eq!(
        measured(source, &body),
        [(6, "handle".to_string()), (6, "handle".to_string())]
    );
}
