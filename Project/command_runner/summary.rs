use crate::project_config::{View, Views};
use crate::shared::console;
use std::{path::Path, process::Output};

pub fn print(name: &str, views: &Views, output: &Output, log: &Path) {
    let code = super::jobs::process::exit_code(output);
    let view = match code {
        0 => &views.success,
        _ => &views.failure,
    };
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let lines: Vec<&str> = stdout.lines().chain(stderr.lines()).collect();
    for line in select(view, &lines) {
        console::stdout(line);
    }
    console::stdout(format!("{name}: exit {code}; log: {}", log.display()));
}
fn select<'a>(view: &View, lines: &[&'a str]) -> Vec<&'a str> {
    let start = lines.len().saturating_sub(view.tail);
    lines
        .iter()
        .enumerate()
        .filter(|(index, line)| {
            *index >= start
                || view
                    .matching
                    .iter()
                    .any(|part| line.contains(part.as_str()))
        })
        .map(|(_, line)| *line)
        .collect()
}
