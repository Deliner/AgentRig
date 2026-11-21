use super::{initialized, lint, text};
use crate::project_setup::init;
use crate::shared::testing::write;
use serde_yaml_ng::Value;
use std::fs;

const MANIFEST: &str = "[package]\nname = \"sample\"\nversion = \"0.1.0\"\n";
const FORMS: [(&str, &[&str]); 3] = [
    (
        "[build]\nrustflags = \"--cfg  probe -Ctarget-cpu=native\"\n",
        &["--cfg", "probe", "-Ctarget-cpu=native"],
    ),
    (
        "[build]\nrustflags = [\"--cfg\", \"probe value\"]\n[target.x86_64-unknown-linux-gnu]\nrustflags = [\"-Cdebuginfo=0\"]\n",
        &["--cfg", "probe value"],
    ),
    ("[net]\noffline = true\n", &[]),
];

// DECISION: D182
#[test]
fn rust_init_copies_build_rustflags_into_the_rig_list_and_keeps_the_cargo_configuration() {
    for (config, flags) in FORMS {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().canonicalize().unwrap();
        write(&root, "Cargo.toml", MANIFEST);
        write(&root, "src/main.rs", "fn main() {}\n");
        write(&root, ".cargo/config.toml", config);
        let args = ["--language", "rust"].map(str::to_owned);
        assert_eq!(init(&root, &args).unwrap(), 0, "{config}");
        let module = fs::read_to_string(root.join(".agentrig/rust.yaml")).unwrap();
        let module: Value = serde_yaml_ng::from_str(&module).unwrap();
        let listed: Vec<&str> = module["rustflags"]
            .as_sequence()
            .unwrap()
            .iter()
            .map(|flag| flag.as_str().unwrap())
            .collect();
        assert_eq!(listed, flags, "{config}");
        let kept = fs::read_to_string(root.join(".cargo/config.toml")).unwrap();
        assert_eq!(kept, config);
    }
}

// DECISION: D182
#[test]
fn shipped_rules_keep_compiler_flags_in_the_rig_list() {
    let (_temp, root) = initialized("rust");
    write(
        &root,
        ".agentrig/rust.yaml",
        "rustflags:\n- --cap-lints\n- allow\n",
    );
    let config = "[build]\nrustflags = [\"--cfg\", \"probe\"]\n";
    write(&root, ".cargo/config.toml", config);
    let (code, found) = lint(&root);
    assert_eq!(code, Some(1), "{found:?}");
    let expected = [
        (
            "allow-justification",
            ".agentrig/rust.yaml",
            "justify-allow",
        ),
        ("cargo-rustflags", ".cargo/config.toml", "move-rustflags"),
    ];
    for (rule, path, skill) in expected {
        let finding = found
            .iter()
            .find(|item| text(item, "rule") == rule && text(item, "path") == path);
        let finding = finding.unwrap_or_else(|| panic!("{rule}: {found:?}"));
        let repair = text(finding, "skill").ends_with(&format!("{skill}/SKILL.md"));
        assert_eq!(
            (text(finding, "level"), repair),
            ("error", true),
            "{finding:?}"
        );
    }
}
