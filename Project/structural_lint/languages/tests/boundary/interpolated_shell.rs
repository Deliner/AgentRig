use super::measured;
use crate::structural_lint::rules::reference::Reference;

#[test]
fn rust_flags_a_shell_command_flag_followed_by_data_anywhere_in_the_chain() {
    let source = r#"fn run(script: &str, path: &str) {
    Command::new("sh").arg("-c").arg(script).status();
    Command::new("/bin/bash")
        .args(["-lc", &format!("ls {path}")])
        .output();
    std::process::Command::new("CMD.exe").arg("/c").current_dir(path).arg(script).spawn();
    Command::new("pwsh").args(&["-NoProfile", "-Command", script]).status();
}
"#;
    assert_eq!(
        measured(source, Reference::InterpolatedShell),
        [
            (2, "sh".to_string()),
            (3, "bash".to_string()),
            (6, "cmd".to_string()),
            (7, "pwsh".to_string())
        ]
    );
}

#[test]
fn rust_skips_literal_and_constant_commands_other_flags_programs_and_order() {
    let source = r#"fn run(path: &str, shell: &str, rest: &[&str]) {
    Command::new("sh").arg("-c").arg("ls -la").status();
    Command::new("sh").args(["-c", SCRIPT]).arg(path).status();
    Command::new("sh").arg("-c").arg(&config::HOOK).status();
    Command::new("bash").arg("-C").arg(path).status();
    Command::new("bash").arg("--rcfile").arg(path).status();
    Command::new("pwsh").arg("-ExecutionPolicy").arg(path).status();
    Command::new("git").arg("-c").arg(path).status();
    Command::new(shell).arg("-c").arg(path).status();
    Command::new("sh").arg(path).arg("-c").status();
    Command::new("sh").args(rest).arg("-c").arg(path).status();
}
"#;
    assert!(measured(source, Reference::InterpolatedShell).is_empty());
}

#[test]
fn rust_skips_a_justified_shell_command_and_test_code() {
    let source = r#"fn run(path: &str) {
    // DECISION: D001
    Command::new("sh").arg("-c").arg(path).status();
}
#[cfg(test)]
mod tests {
    fn run(path: &str) {
        Command::new("sh").arg("-c").arg(path).status();
    }
}
"#;
    assert!(measured(source, Reference::InterpolatedShell).is_empty());
}
