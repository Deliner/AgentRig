use super::architecture::binaries;
use super::consumer::{CONFIG, REPAIR, SKILL, parsed, prepare};
use crate::shared::testing::{git, hermetic, run, snapshot, worker};
use std::{
    fs,
    path::{Path, PathBuf},
};
use tempfile::TempDir;

fn portable(config: &str) -> (TempDir, PathBuf) {
    let temp = tempfile::tempdir().unwrap();
    let base = temp.path().canonicalize().unwrap();
    prepare(
        &base.join("policy"),
        &format!("skill_root: \".\"\n{config}"),
    );
    fs::create_dir_all(base.join("consumer")).unwrap();
    (temp, base)
}

fn skill(policy: &Path, name: &str) -> String {
    policy.join(name).to_str().unwrap().to_owned()
}

#[test]
fn standalone_matches_worker_without_installing_files() {
    for versioned in [false, true] {
        let (_temp, base) = portable(CONFIG);
        let (policy, project) = (base.join("policy"), base.join("consumer"));
        fs::create_dir_all(project.join("src")).unwrap();
        fs::write(project.join("src/value.py"), "line\n".repeat(6)).unwrap();
        if versioned {
            git(&project, &["init", "-q"]);
        }
        let before = snapshot(&project);
        let results = binaries().map(|binary| {
            let mut command = hermetic(&binary);
            let integrated = binary == worker();
            if integrated {
                command.arg("lint");
            }
            command.arg("--root").arg(&project);
            command.arg("--config").arg(policy.join("lint.yaml"));
            let result = run(command.arg("--json").current_dir(&base), "");
            assert_eq!(result.code, Some(1), "{}", result.output());
            let mut findings = parsed(&result);
            assert_eq!(findings[0]["skill"], skill(&policy, SKILL));
            for finding in &mut findings {
                let rerun = finding.as_object_mut().unwrap().remove("rerun").unwrap();
                let named = rerun.as_str().unwrap().contains(binary.to_str().unwrap());
                assert!(named, "{rerun}");
            }
            findings
        });
        assert_eq!(results[0], results[1]);
        assert_eq!(snapshot(&project), before);
    }
}

#[test]
fn standalone_config_check_rejects_unsupported_language() {
    let config = CONFIG.replace("kind: \"nonblank-lines\"", "kind: \"function-lines\"");
    let (_temp, base) = portable(&config);
    let (policy, project) = (base.join("policy"), base.join("consumer"));
    let [_, standalone] = binaries();
    let mut command = hermetic(&standalone);
    command.arg("lint-config-check").arg("--root").arg(&project);
    command.arg("--config").arg(policy.join("lint.yaml"));
    let result = run(command.arg("--json").current_dir(&base), "");
    assert_eq!(result.code, Some(2), "{}", result.output());
    let diagnostic = &parsed(&result)[0];
    let message = diagnostic["message"].as_str().unwrap();
    assert!(message.contains("incompatible"), "{message}");
    assert_eq!(diagnostic["skill"], skill(&policy, REPAIR));
    assert_eq!(fs::read_dir(&project).unwrap().count(), 0);
}
