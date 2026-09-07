use review_runner::vcs::{Backend, Settings, external::Adapter};
use serde_json::{Value, json};
use std::fs;

fn settings(context: Value, observation: Value) -> Settings {
    let script = r#"
import json, os, pathlib, sys, tempfile
request = json.load(sys.stdin)
context, observation = json.loads(sys.argv[1])
operation = request['operation']
result = None
if operation == 'prepare-integration':
    assert request['arguments'] == {'base': 'main', 'prefix': 'task/'}
    result = context
elif operation == 'finish-integration':
    assert request['arguments'] == {'policy': {'base': 'main', 'prefix': 'task/'}, 'expected': context}
    with tempfile.NamedTemporaryFile() as temporary:
        temporary.write(b'hook export')
    pathlib.Path('finished').write_text(os.environ['TMPDIR'])
elif operation == 'observe':
    result = observation
else:
    sys.exit(64)
json.dump({'version': 1, 'result': result}, sys.stdout)
"#;
    Settings {
        backend: Backend::External(Adapter {
            command: vec![
                "python3".into(),
                "-c".into(),
                script.into(),
                json!([context, observation]).to_string(),
            ],
        }),
        base: "main".into(),
        prefix: "task/".into(),
    }
}

fn context() -> Value {
    json!({"feature": "task/example", "base": "base-42", "candidate": "candidate-43"})
}

fn observation() -> Value {
    json!({"branch": "main", "revision": "merged-44", "status": "", "merge_in_progress": false, "rebase_in_progress": false})
}

#[test]
fn malformed_integration_context_never_runs_checks_or_finishes() {
    for invalid in [
        json!({"feature": "main", "base": "base-42", "candidate": "candidate-43"}),
        json!({"feature": "task/example", "base": "", "candidate": "candidate-43"}),
        json!({"feature": "task/example", "base": "base-42", "candidate": "bad\nrevision"}),
        json!({"feature": "task/example", "base": "base-42", "candidate": 43}),
        json!({"feature": "task/example", "base": "base-42", "candidate": "candidate-43", "extra": true}),
    ] {
        let directory = tempfile::tempdir().unwrap();
        let settings = settings(invalid, observation());
        let result = settings.backend.source(directory.path()).integrate(
            &settings,
            || panic!("invalid context reached checks"),
            |_| panic!("native fallback"),
        );
        assert!(result.is_err());
        assert!(!directory.path().join("finished").exists());
    }
}

#[test]
fn failed_gate_does_not_finish_private_integration() {
    let directory = tempfile::tempdir().unwrap();
    let settings = settings(context(), observation());
    let result = settings
        .backend
        .source(directory.path())
        .integrate(&settings, || Ok(17), |_| panic!("native fallback"))
        .unwrap();
    assert_eq!(result, ("task/example".into(), 17));
    assert!(!directory.path().join("finished").exists());
}

#[test]
fn completion_requires_clean_base_and_cleans_hook_scratch() {
    let mut dirty = observation();
    dirty["status"] = json!("M source");
    for (observed, succeeds) in [(observation(), true), (dirty, false)] {
        let directory = tempfile::tempdir().unwrap();
        let settings = settings(context(), observed);
        let result = settings.backend.source(directory.path()).integrate(
            &settings,
            || Ok(0),
            |_| panic!("native fallback"),
        );
        assert_eq!(result.is_ok(), succeeds);
        let scratch = fs::read_to_string(directory.path().join("finished")).unwrap();
        assert!(!std::path::Path::new(&scratch).exists());
    }
}
