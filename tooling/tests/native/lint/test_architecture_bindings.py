from pathlib import Path

import pytest
from test_architecture import consumer, contract
from test_architecture_behavior import parity, run_consumer


@pytest.mark.parametrize("form", ["impl", "macro", "trait", "nested", "outside"])
def test_self_binding_uses_its_lexical_owner(worker: Path, tmp_path: Path, form: str) -> None:
    source, _ = consumer(tmp_path, "rs")
    bodies = {
        "impl": "Self::value()",
        "macro": "{ assert_eq!(Self::value(), 7); 7 }",
        "trait": "Self::value()",
        "nested": "{ fn nested() -> i32 { Self::value() } nested() }",
        "outside": "Self::value()",
    }
    trait_owner = form == "trait"
    declaration = "pub trait Behavior" if trait_owner else "pub struct Value; impl Value"
    code = (
        f"{declaration} {{ fn value() -> i32 {{ crate::b::value() }} "
        f"fn call() -> i32 {{ {bodies[form]} }} }} "
        "pub fn run() -> i32 { Value::call() }"
    )
    if trait_owner:
        code += " pub struct Value; impl Behavior for Value {}"
    missing_owner = form == "outside"
    if missing_owner:
        code = "pub fn run() -> i32 { Self::value() }"
    source.write_text(code)
    unsupported = form in {"nested", "outside"}
    if unsupported:
        status, findings = parity(worker, tmp_path)
        assert status == 1 and any("incomplete" in item["message"] for item in findings)
        return
    entry = tmp_path / "src/lib.rs"
    entry.write_text(entry.read_text() + "fn main() { std::process::exit(a::run()); }")
    assert run_consumer(tmp_path, "rs") == "7"
    assert parity(worker, tmp_path) == (0, [])
    contract(tmp_path / "src/a")
    status, findings = parity(worker, tmp_path)
    assert status == 1 and any("forbidden" in item["message"] for item in findings)


@pytest.mark.parametrize("form", ["standard", "import", "generic", "local", "wildcard"])
def test_prelude_names_preserve_local_bindings(worker: Path, tmp_path: Path, form: str) -> None:
    source, _ = consumer(tmp_path, "rs")
    samples = {
        "standard": "pub fn run() -> i32 { let _ = String::new(); let _ = Vec::<u8>::new(); i32::from(7u8) }",
        "import": "use crate::b as String; pub fn run() -> i32 { String::value() }",
        "generic": "pub fn run<String>() { String::value(); }",
        "local": "pub fn run() { struct String; String::value(); }",
        "wildcard": "use unknown::*; pub fn run() { String::value(); }",
    }
    source.write_text(samples[form])
    unsupported = form in {"generic", "local", "wildcard"}
    status, findings = parity(worker, tmp_path)
    if unsupported:
        assert status == 1 and any("incomplete" in item["message"] for item in findings), findings
        return
    assert status == 0, findings
    entry = tmp_path / "src/lib.rs"
    entry.write_text(entry.read_text() + "fn main() { std::process::exit(a::run()); }")
    assert run_consumer(tmp_path, "rs") == "7"
    imported = form == "import"
    if imported:
        contract(tmp_path / "src/a")
        status, findings = parity(worker, tmp_path)
        assert status == 1 and any("forbidden" in item["message"] for item in findings)


@pytest.mark.parametrize("bound", ["crate::b::Access", "crate::b::Access + Send", ""])
def test_generic_paths_require_one_explicit_bound(worker: Path, tmp_path: Path, bound: str) -> None:
    source, target = consumer(tmp_path, "rs")
    target.write_text(
        "pub trait Access { fn value() -> i32 { 7 } } pub struct Value; impl Access for Value {}"
    )
    declaration = f"D: {bound}" if bound else "D"
    source.write_text(
        f"fn call<{declaration}>() -> i32 {{ D::value() }} "
        "pub fn run() -> i32 { call::<crate::b::Value>() }"
    )
    status, findings = parity(worker, tmp_path)
    supported = bound == "crate::b::Access"
    if supported:
        assert status == 0, findings
        entry = tmp_path / "src/lib.rs"
        entry.write_text(entry.read_text() + "fn main() { std::process::exit(a::run()); }")
        assert run_consumer(tmp_path, "rs") == "7"
        contract(tmp_path / "src/a")
        status, findings = parity(worker, tmp_path)
        assert status == 1 and any("forbidden" in item["message"] for item in findings)
    else:
        assert status == 1 and any("incomplete" in item["message"] for item in findings)


def test_local_item_shadows_generic_bound(worker: Path, tmp_path: Path) -> None:
    source, _ = consumer(tmp_path, "rs")
    source.write_text("fn call<D: external::Access>() { struct D; D::value(); }")
    status, findings = parity(worker, tmp_path)
    assert status == 1 and any("unresolved Rust name a::D" in item["message"] for item in findings)
