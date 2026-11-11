use super::{Consumer, Language, described, rust};

const POLICY: &str = "      disallowed:
        - path: std::fs
          reason: filesystem access belongs to the owner
          replacement: crate::b
          allow: ['src/b/**']
";
const REASON: &str = "is disallowed here: filesystem access belongs to the owner; use crate::b";
const METHOD_CALLS: &str =
    "pub fn run(path: &std::path::Path, world: &str) -> std::io::Result<i32> {
    let _ = path.canonicalize()?;
    let _ = path
        .metadata()
        .map_err(Into::into)?;
    let _ = path.metadata().map_err(std::io::Error::other)?;
    let _ = path.metadata().ok();
    let _ = path.canonicalize();
    if let Ok(found) = path.metadata() {}
    let _ = world.metadata(1)?;
    let _ = path.read_link()?;
    assert!(path.starts_with(path.canonicalize()?));
    assert!(path.metadata().is_ok());
    assert!(path.metadata().map_err(Into::into)?.is_file());
    Ok(0)
}
";
const BROKEN: [(&str, &str); 9] = [
    (
        "allow: ['src/b/**']",
        "allow: ['src/b/**']\n          methods: ['']",
    ),
    (
        "allow: ['src/b/**']",
        "allow: ['src/b/**']\n          methods: [path.exists]",
    ),
    ("path: std::fs", "path: 'std::*'"),
    ("path: std::fs", "path: os.path"),
    (
        "reason: filesystem access belongs to the owner",
        "reason: ''",
    ),
    ("replacement: crate::b", "replacement: ''"),
    ("allow: ['src/b/**']", "allow: ['../b/**']"),
    ("allow: ['src/b/**']", "allow: ['src/[b/**']"),
    ("allow: ['src/b/**']", "allow: []\n          extra: true"),
];

fn policed(policy: &str) -> Consumer {
    let consumer = Consumer::new(Language::Rust);
    consumer.write("src/a/fs.rs", "pub fn local() -> i32 { 1 }\n");
    consumer.source(
        "pub mod fs;\npub fn run() -> i32 {\n    let _ = std::fs::read_to_string(\"x\");\n    \
         fs::local() + crate::b::value()\n}\n",
    );
    consumer.target("pub fn value() -> i32 { std::fs::metadata(\"x\").is_ok() as i32 }\n");
    consumer.contract("src/a");
    consumer.configure(
        "rust_roots: [src/lib.rs]\n",
        &format!("rust_roots: [src/lib.rs]\n{policy}"),
    );
    consumer
}

// DECISION: D105
#[test]
fn disallowed_paths_flag_references_outside_allowed_files() {
    let message = format!("`std::fs::read_to_string` {REASON}");
    assert_eq!(
        policed(POLICY).located(),
        [("src/a/mod.rs".to_owned(), Some(3), message)]
    );
    assert!(described().get("disallowed").is_some());
}

// DECISION: D125
#[test]
fn disallowed_methods_flag_bare_calls_whose_errors_propagate() {
    let consumer = rust(METHOD_CALLS);
    consumer.target(&METHOD_CALLS.replace("run", "value"));
    let methods =
        format!("rust_roots: [src/lib.rs]\n{POLICY}          methods: [canonicalize, metadata]\n");
    consumer.configure("rust_roots: [src/lib.rs]\n", &methods);
    let expected = [
        (2, "canonicalize"),
        (4, "metadata"),
        (12, "canonicalize"),
        (14, "metadata"),
    ]
    .map(|(line, method)| {
        let message = format!("`.{method}()` with its error propagated {REASON}");
        ("src/a/mod.rs".to_owned(), Some(line), message)
    });
    assert_eq!(consumer.located(), expected);
    let fields = described()["disallowed"].to_string();
    assert!(fields.contains("methods"), "{fields}");
}

// DECISION: D105
#[test]
fn disallowed_config_check_rejects_invalid_entries() {
    for (from, to) in BROKEN {
        policed(&POLICY.replacen(from, to, 1)).invalid();
    }
}
