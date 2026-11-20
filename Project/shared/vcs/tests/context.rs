use crate::shared::vcs::{Repository, git_context};
use std::{path::Path, process::Command};

const CONTEXT_ROOT: &str = "AGENTRIG_TEST_GIT_CONTEXT_ROOT";

fn initialize(root: &Path, branch: &str) {
    assert!(
        Command::new("git")
            .args(["init", "-q", "-b", branch])
            .arg(root)
            .env_remove("GIT_DIR")
            .env_remove("GIT_WORK_TREE")
            .status()
            .unwrap()
            .success()
    );
}

#[test]
fn query_inherits_repository_context() {
    let args = ["branch", "--show-current"];
    if let Ok(root) = std::env::var(CONTEXT_ROOT) {
        let root = Path::new(&root);
        assert_eq!(git_context(root, &args).unwrap(), "inherited");
        let isolated = Repository::new(root).run(&args).unwrap();
        assert_eq!(String::from_utf8(isolated).unwrap().trim(), "local");
        return;
    }
    let root = tempfile::tempdir().unwrap();
    let inherited = tempfile::tempdir().unwrap();
    for (path, branch) in [(root.path(), "local"), (inherited.path(), "inherited")] {
        initialize(path, branch);
    }
    let (_, module) = module_path!().split_once("::").unwrap();
    let name = format!("{module}::query_inherits_repository_context");
    let output = Command::new(std::env::current_exe().unwrap())
        .args(["--exact", &name])
        .env(CONTEXT_ROOT, root.path())
        .env("GIT_DIR", inherited.path().join(".git"))
        .env("GIT_WORK_TREE", inherited.path())
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    let report = format!("{stdout}\n{}", String::from_utf8_lossy(&output.stderr));
    assert!(output.status.success(), "{report}");
    assert!(stdout.contains("test result: ok. 1 passed"), "{report}");
}

#[test]
fn query_failure_preserves_git_stderr() {
    let root = tempfile::tempdir().unwrap();
    let args = ["--not-an-agentrig-option"];
    let expected = Command::new("git")
        .args(args)
        .current_dir(root.path())
        .output()
        .unwrap();
    assert!(!expected.status.success());
    assert_eq!(
        git_context(root.path(), &args).unwrap_err().to_string(),
        String::from_utf8_lossy(&expected.stderr)
    );
}
