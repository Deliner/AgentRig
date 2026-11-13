use super::write;
use crate::project_setup::initialization::init;
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
        let args = ["--language", "rust", "--source", "src"].map(str::to_owned);
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
