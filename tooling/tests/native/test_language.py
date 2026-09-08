from __future__ import annotations

from pathlib import Path

import pytest

from tooling.tests.native.test_lint import CONFIG, lint, prepare

# DECISION: D017


def configure(root: Path, kind: str, thresholds: str = 'level: "error"') -> None:
    config = (
        CONFIG.replace("nonblank-lines", kind)
        .replace('extensions: [".rs", ".py", ".ts"]', 'extensions: [".rs", ".py", ".pyi"]')
        .replace("warning: 3\n    error: 5", thresholds.replace("\n", "\n    "))
    )
    prepare(root, config)


@pytest.mark.parametrize(
    "example",
    [
        (
            "rs",
            """fn run() {
    if ready {}
    if (/* reason */ state.ready) {}
    if ready && enabled {}
    if value > 3 {}
    if ready() {}
    if !ready {}
    if let Some(value) = input {}
    if let Some(value) = input && value > 0 {}
    if crate::READY {}
    // if fake && condition {}
    let text = "if fake && condition {}";
    if factory().ready {}
    if values[0] {}
}
""",
            [4, 5, 6, 7, 9, 13, 14],
        ),
        (
            "py",
            """if ready:
    pass
elif state.ready:
    pass
elif ready and enabled:
    pass
if value > 3:
    pass
if ready():
    pass
if not ready:
    pass
result = yes if (ready) else no
result = yes if ready and enabled else no
items = [x for x in values if x > 0]
# if fake and condition:
text = "if fake and condition:"
if factory().ready:
    pass
""",
            [5, 7, 9, 11, 14, 15, 18],
        ),
    ],
)
# INVARIANT: I015
def test_language_conditions(
    worker: Path, tmp_path: Path, example: tuple[str, str, list[int]]
) -> None:
    extension, source, lines = example
    configure(tmp_path, "named-if-condition")
    (tmp_path / f"src/example.{extension}").write_text(source, encoding="utf-8")
    code, items = lint(worker, tmp_path)
    assert code == 1
    assert [item["line"] for item in items] == lines
    assert all(item["rule"] == "source" and item["level"] == "error" for item in items)
    assert all(item["skill"].endswith("SKILL.md") for item in items)
    path = tmp_path / "lint.yaml"
    path.write_text(path.read_text().replace('level: "error"', 'level: "warning"'))
    code, items = lint(worker, tmp_path)
    assert code == 0
    assert all(item["level"] == "warning" for item in items)


@pytest.mark.parametrize(
    "example",
    [
        (
            "rs",
            """trait Worker {
    fn declared(&self, one: i32, two: i32);
}
impl WorkerImpl {
    fn new(one: i32, pair: (i32, i32), callback: fn(i32, i32)) -> Self { todo!() }
    fn run(&mut self, value: Vec<(i32, i32)>) {}
    fn owned(self: Box<Self>, value: i32) {}
}
fn outer() { let closure = |one, two: (i32, i32)| one; }
""",
            {"declared": 2, "new": 3, "run": 1, "owned": 1, "<anonymous>": 2},
        ),
        (
            "py",
            """class Worker:
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
""",
            {
                "__init__": 3,
                "create": 1,
                "static": 2,
                "method": 3,
                "nested": 2,
                "outer": 2,
                "<anonymous>": 2,
            },
        ),
    ],
)
def test_parameters_are_declarations(
    worker: Path, tmp_path: Path, example: tuple[str, str, dict[str, int]]
) -> None:
    extension, source, counts = example
    configure(tmp_path, "parameter-count", "warning: 0")
    (tmp_path / f"src/example.{extension}").write_text(source)
    code, items = lint(worker, tmp_path)
    assert code == 0
    assert {item["symbol"]: item["actual"] for item in items} == counts


@pytest.mark.parametrize(
    "example",
    [
        (
            "rs",
            """fn long(
    one: i32,
) {
    // comment

    let text = "}";
    if ready {
        work();
    }
}
fn short() {}
""",
            9,
        ),
        (
            "py",
            """@decorator
async def long(
    one: int,
):
    # comment

    text = "def fake():"
    def nested():
        pass
    return one
def short(): pass
""",
            8,
        ),
    ],
)
def test_function_ranges(worker: Path, tmp_path: Path, example: tuple[str, str, int]) -> None:
    extension, source, count = example
    configure(tmp_path, "function-lines", f"warning: {count - 1}\nerror: {count}")
    (tmp_path / f"src/example.{extension}").write_text(source)
    code, items = lint(worker, tmp_path)
    assert code == 0
    assert [(item["symbol"], item["actual"], item["level"]) for item in items] == [
        ("long", count, "warning")
    ]
    path = tmp_path / "lint.yaml"
    path.write_text(
        path.read_text().replace(f"warning: {count - 1}\n    error: {count}", f"error: {count - 1}")
    )
    assert lint(worker, tmp_path)[0] == 1


@pytest.mark.parametrize(
    ("extension", "source"), [("rs", "fn broken( { if }"), ("py", "def broken(:\n pass")]
)
def test_malformed_source_blocks(worker: Path, tmp_path: Path, extension: str, source: str) -> None:
    configure(tmp_path, "named-if-condition", 'level: "warning"')
    (tmp_path / f"src/broken.{extension}").write_text(source)
    code, items = lint(worker, tmp_path)
    assert code == 1
    assert items[0]["rule"] == "syntax"
    assert items[0]["line"] >= 1
    assert items[0]["path"] == f"src/broken.{extension}"


def test_language_selection_and_overrides(worker: Path, tmp_path: Path) -> None:
    configure(tmp_path, "parameter-count", "warning: 1\nerror: 3")
    (tmp_path / "src/example.py").write_text("def example(one, two, three): pass")
    (tmp_path / "src/example.js").write_text("not valid Python or Rust")
    config = tmp_path / "lint.yaml"
    config.write_text(
        config.read_text() + '\n    overrides:\n      - include: ["src/*.py"]\n        error: 2\n'
    )
    code, items = lint(worker, tmp_path)
    assert code == 1
    assert items[0]["actual"] == 3
    assert items[0]["limit"] == 2
    config.write_text(config.read_text().replace('[".rs", ".py", ".pyi"]', '[".js"]'))
    assert lint(worker, tmp_path)[0] == 2


@pytest.mark.parametrize("policy", ["warning: 1", 'level: "fatal"', 'level: "error"\nwarning: 1'])
def test_condition_configuration_is_not_numeric(worker: Path, tmp_path: Path, policy: str) -> None:
    configure(tmp_path, "named-if-condition", policy)
    assert lint(worker, tmp_path)[0] == 2
