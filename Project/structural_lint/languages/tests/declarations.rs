// DECISION: D017
use super::symbol_actuals;
use crate::structural_lint::rule_identities::kind::Kind;
use crate::structural_lint::rule_identities::signature::Signature;
use crate::structural_lint::rule_identities::structure::Structure;

const PARAMETERS: Kind = Kind::Signature(Signature::ParameterCount);
const LINES: Kind = Kind::Structure(Structure::FunctionLines);

#[test]
fn rust_parameters_are_declarations() {
    let source = "trait Worker {
    fn declared(&self, one: i32, two: i32);
}
impl WorkerImpl {
    fn new(one: i32, pair: (i32, i32), callback: fn(i32, i32)) -> Self { todo!() }
    fn run(&mut self, value: Vec<(i32, i32)>) {}
    fn owned(self: Box<Self>, value: i32) {}
}
fn outer() { let closure = |one, two: (i32, i32)| one; }
";
    assert_eq!(
        symbol_actuals(PARAMETERS, "source.rs", source),
        [
            ("<anonymous>".into(), 2),
            ("declared".into(), 2),
            ("new".into(), 3),
            ("outer".into(), 0),
            ("owned".into(), 1),
            ("run".into(), 1),
        ]
    );
}

#[test]
fn python_parameters_are_declarations() {
    let source = "class Worker:
    def __init__(self, one, two=(1, 2), *, three: tuple[int, int]):
        pass
    @classmethod
    def create(cls, one):
        pass
    @staticmethod
    def static(self, two):
        pass
    def method(this, one, /, *args, **kwargs):
        def nested(self, value):
            pass
        pass
async def outer(self, value: tuple[int, int] = (1, 2)):
    pass
callback = lambda one, two: one
";
    assert_eq!(
        symbol_actuals(PARAMETERS, "source.py", source),
        [
            ("<anonymous>".into(), 2),
            ("__init__".into(), 3),
            ("create".into(), 1),
            ("method".into(), 3),
            ("nested".into(), 2),
            ("outer".into(), 2),
            ("static".into(), 2),
        ]
    );
}

#[test]
fn rust_function_ranges() {
    let source = "fn long(
    one: i32,
) {
    // comment

    let text = \"}\";
    if ready {
        work();
    }
}
fn short() {}
";
    assert_eq!(
        symbol_actuals(LINES, "source.rs", source),
        [("long".into(), 9), ("short".into(), 1)]
    );
}

#[test]
fn python_function_ranges() {
    let source = "@decorator
async def long(
    one: int,
):
    # comment

    text = \"def fake():\"
    def nested():
        pass
    return one
def short(): pass
";
    assert_eq!(
        symbol_actuals(LINES, "source.py", source),
        [
            ("long".into(), 8),
            ("nested".into(), 2),
            ("short".into(), 1)
        ]
    );
}
