use review_runner::vcs::{Backend, Generation, Kind, external::Adapter};
use serde_json::json;
use std::{fs, path::Path, process::Command};

fn external() -> Backend {
    // Cargo supplies the current crate path even when the test binary is cached.
    let script = std::path::PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR").unwrap())
        .join("../examples/external_vcs.py");
    Backend::External(Adapter {
        command: vec![
            "python3".into(),
            "-B".into(),
            script.to_str().unwrap().into(),
        ],
    })
}

#[test]
fn private_registration_preserves_custom_configuration_and_runs_native_hook() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path();
    let backend = external();
    let hooks = ".agentrig/hooks with spaces";
    backend.source(root).validate_registration(hooks).unwrap();
    assert!(!backend.source(root).hooks_registered(hooks).unwrap());
    assert!(!root.join(".hg").exists());
    backend.initialize(root, "main").unwrap();
    let config = root.join(".hg/hgrc");
    let custom = "[ui]\nusername = Consumer\n[hooks]\npretxncommit.custom = true\n";
    fs::write(&config, custom).unwrap();
    fs::write(root.join(".hgignore"), "syntax: glob\nconsumer-cache/**\n").unwrap();
    let source = backend.source(root);
    source.validate_registration(hooks).unwrap();
    assert!(!source.hooks_registered(hooks).unwrap());
    assert_eq!(fs::read_to_string(&config).unwrap(), custom);
    source.register_hooks(hooks).unwrap();
    assert!(source.hooks_registered(hooks).unwrap());
    let registered = fs::read_to_string(&config).unwrap();
    let ignored = fs::read_to_string(root.join(".hgignore")).unwrap();
    assert_eq!(
        ignored,
        format!("syntax: glob\nconsumer-cache/**\n\ninclude:{hooks}.hgignore\n")
    );
    assert!(registered.starts_with(custom));
    source.register_hooks(hooks).unwrap();
    assert_eq!(fs::read_to_string(&config).unwrap(), registered);
    assert_eq!(fs::read_to_string(root.join(".hgignore")).unwrap(), ignored);
    let native = Backend::Native(Kind::Mercurial);
    assert!(native.source(root).hooks_registered(hooks).unwrap());
    run_registered_hook(root, hooks);
}

fn run_registered_hook(root: &Path, hooks: &str) {
    fs::create_dir_all(root.join(hooks)).unwrap();
    fs::write(
        root.join(format!("{hooks}.hgignore")),
        "syntax: glob\nhook-called\n",
    )
    .unwrap();
    fs::write(
        root.join(hooks).join("pretxncommit"),
        "printf called > hook-called\n",
    )
    .unwrap();
    for args in [vec!["addremove"], vec!["commit", "-m", "consumer"]] {
        let output = Command::new("hg")
            .args(args)
            .current_dir(root)
            .env("HGPLAIN", "1")
            .env("HGRCPATH", "")
            .env_remove("HGRCSKIPREPO")
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    assert_eq!(
        fs::read_to_string(root.join("hook-called")).unwrap(),
        "called"
    );
}

#[test]
fn private_registration_rejects_each_conflict_without_overwriting_configuration() {
    for setting in [
        "[hooks]\npretxncommit.agentrig = custom\n",
        "[ui]\nignore.agentrig = custom\n",
    ] {
        let directory = tempfile::tempdir().unwrap();
        let root = directory.path();
        let backend = external();
        backend.initialize(root, "main").unwrap();
        let config = root.join(".hg/hgrc");
        fs::write(&config, setting).unwrap();
        let source = backend.source(root);
        let error = source.register_hooks(".agentrig/hooks").unwrap_err();
        assert!(error.to_string().contains("setup conflict"));
        assert_eq!(fs::read_to_string(config).unwrap(), setting);
    }
}

#[test]
fn registration_rejects_invalid_descriptions_and_false_write_success() {
    let root = tempfile::tempdir().unwrap();
    for values in [
        json!([]),
        json!([["key", "", ""]]),
        json!([["key", "", "value"], ["key", "", "value"]]),
    ] {
        let backend = registration_reply(values);
        assert!(
            backend
                .source(root.path())
                .validate_registration("hooks")
                .is_err()
        );
    }
    let backend = registration_reply(json!([["key", "", "value"]]));
    let source = backend.source(root.path());
    source.validate_registration("hooks").unwrap();
    assert!(
        source
            .register_hooks("hooks")
            .unwrap_err()
            .to_string()
            .contains("did not establish")
    );
}

fn registration_reply(values: serde_json::Value) -> Backend {
    let script = format!(
        "import json,sys; r=json.load(sys.stdin); v=json.loads({:?}); result=v if r['operation']=='registration' else None; print(json.dumps({{'version':1,'result':result}}))",
        values.to_string()
    );
    Backend::External(Adapter {
        command: vec!["python3".into(), "-c".into(), script],
    })
}

#[test]
fn private_generation_matches_native_mercurial_without_writing_files() {
    let root = tempfile::tempdir().unwrap();
    let request = Generation {
        binary: "\"$root\"/'rig space/bin/agentrig'",
        directory: "rig space/hooks",
        ignored: &["runtime".into(), "rig space/review/runtime".into()],
    };
    let native = Backend::Native(Kind::Mercurial)
        .generate(root.path(), &request)
        .unwrap();
    let private = external().generate(root.path(), &request).unwrap();
    assert_eq!(private.root_command, native.root_command);
    assert_eq!(private.files, native.files);
    assert_eq!(fs::read_dir(root.path()).unwrap().count(), 0);
}

#[test]
fn generation_rejects_files_outside_the_hook_area_and_invalid_root_commands() {
    let root = tempfile::tempdir().unwrap();
    let request = Generation {
        binary: "worker",
        directory: ".agentrig/hooks",
        ignored: &[],
    };
    for path in [
        "../escape",
        "/tmp/escape",
        ".git/config",
        "AGENTS.md",
        ".agentrig/bin/agentrig",
        ".agentrig/hooks",
        ".agentrig/hooks.extra/child",
    ] {
        let value = json!({"root_command":"hg root", "files":{path:"contents"}});
        assert!(
            generation_reply(value)
                .generate(root.path(), &request)
                .is_err()
        );
    }
    for command in [json!(""), json!("bad\u{0}command"), json!(42)] {
        let value = json!({"root_command":command, "files":{}});
        assert!(
            generation_reply(value)
                .generate(root.path(), &request)
                .is_err()
        );
    }
    assert_eq!(fs::read_dir(root.path()).unwrap().count(), 0);
}

fn generation_reply(value: serde_json::Value) -> Backend {
    Backend::External(Adapter {
        command: vec![
            "python3".into(),
            "-c".into(),
            format!(
                "import sys; sys.stdout.write({:?})",
                json!({"version":1,"result":value}).to_string()
            ),
        ],
    })
}
