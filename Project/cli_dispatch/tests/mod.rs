use crate::shared::testing::{FAIL, GATE, REPAIR, hermetic, launch, project, rerun, run, worker};
use std::fs;

#[test]
fn standalone_failure_has_executable_guidance() {
    let unknown = GATE.replace("command: fail", "command: unknown");
    let failures: [(&[&str], &str); 3] = [
        (&["memory-check"], GATE),
        (&["config-check"], &unknown),
        (&["run", "fail"], GATE),
    ];
    for (args, checks) in failures {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().canonicalize().unwrap();
        project(&root, FAIL, checks);
        let result = run(hermetic(worker()).args(args).current_dir(&root), "");
        assert_ne!(result.code, Some(0), "{args:?}: {}", result.output());
        assert!(
            result.stderr.contains(REPAIR),
            "{args:?}: {}",
            result.stderr
        );
        let repeated = rerun(&result.stderr);
        assert_eq!(
            repeated.code,
            result.code,
            "{args:?}: {}",
            repeated.output()
        );
    }
}

#[test]
fn retry_survives_running_binary_replacement() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().canonicalize().unwrap();
    let replacing = "'cp worker worker.next; mv worker.next worker; exit 23'";
    project(&root, &FAIL.replace("exit 23", replacing), GATE);
    let executable = root.join("worker");
    fs::copy(worker(), &executable).unwrap();
    let result = run(&mut launch(&executable, &root, &["run", "fail"]), "");
    assert_eq!(result.code, Some(23), "{}", result.output());
    let repeated = rerun(&result.stderr);
    assert_eq!(repeated.code, Some(23), "{}", repeated.output());
}
