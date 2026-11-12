use super::architecture::binaries;
use super::consumer::{CONFIG, command_with, consumer, write};

const TREE: &str = "src
├── a.rs
├── nested
│   ├── b.rs
│   └── deeper
│       └── c.rs
└── z.rs
";

#[test]
fn tree_prints_names_with_box_drawing() {
    for binary in binaries() {
        let (_temp, root) = consumer(&format!("exclude: [\"src/hidden.py\"]\n{CONFIG}"));
        for name in [
            "src/z.rs",
            "src/a.rs",
            "src/nested/b.rs",
            "src/nested/deeper/c.rs",
            "src/hidden.py",
        ] {
            write(&root, name, "line\n");
        }
        let result = command_with(&binary, &root, "lint-tree", &["./src"]);
        assert_eq!(result.code, Some(0), "{}", result.output());
        assert_eq!(result.stdout, TREE);
    }
}

#[test]
fn tree_rejects_paths_outside_the_directory_inventory() {
    let (_temp, root) = consumer(&format!("exclude: [\"src/hidden\"]\n{CONFIG}"));
    write(&root, "src/a.rs", "line\n");
    write(&root, "src/hidden/b.rs", "line\n");
    for binary in binaries() {
        for path in ["src/a.rs", "src/hidden", "missing", "../src"] {
            let result = command_with(&binary, &root, "lint-tree", &[path]);
            assert_eq!(result.code, Some(2), "{path}: {}", result.output());
            assert!(result.stdout.is_empty(), "{path}: {}", result.output());
        }
        let result = command_with(&binary, &root, "lint-tree", &["src", "--json"]);
        assert_eq!(result.code, Some(2), "{}", result.output());
    }
}
