use crate::project_config::Config;

type Recipe = (&'static str, &'static str, bool);
// DECISION: D039
// DECISION: D189
const RECIPES: [(&str, &[Recipe]); 5] = [
    (
        "commands",
        &[
            ("list", "commands", false),
            ("run", "run", true),
            ("read", "run read", true),
            ("write", "run write", true),
            ("scratch", "scratch", true),
            ("test", "run test", true),
        ],
    ),
    (
        "checks",
        &[
            ("check", "check", true),
            ("config-check", "config-check", false),
        ],
    ),
    (
        "delivery",
        &[
            ("resume", "resume", false),
            ("report", "report", false),
            ("feature-start", "feature-start", true),
            ("feature-merge", "feature-merge", false),
        ],
    ),
    (
        "lint",
        &[
            ("lint", "lint", true),
            ("lint-rules", "lint-rules", false),
            ("lint-config-check", "lint-config-check", true),
            ("lint-baseline", "lint-baseline", true),
            ("lint-rule", "lint-rule", true),
            ("lint-explain", "lint-explain", true),
            ("lint-edges", "lint-edges", true),
            ("lint-tree", "lint-tree", true),
            ("lint-structure", "lint-structure", true),
        ],
    ),
    (
        "jobs",
        &[
            ("jobs", "jobs", false),
            ("job-status", "job-status", true),
            ("job-logs", "job-logs", true),
            ("job-start", "job-start", true),
            ("job-stop", "job-stop", true),
            ("job-cleanup", "job-cleanup", true),
        ],
    ),
];
pub(super) struct Justfile {
    pub path: String,
    pub source: String,
}
pub(super) fn justfiles(config: &Config) -> Vec<Justfile> {
    let mut root = "set positional-arguments := true\n\n".to_owned();
    let path = config.paths.service_path("bin/agentrig");
    let binary = shell_words::quote(&path);
    let mut files = Vec::new();
    for (group, recipes) in RECIPES {
        let file = format!("just/{group}.just");
        root.push_str(&format!("import '{file}'\n"));
        let source: Vec<String> = recipes
            .iter()
            .map(|&(name, command, args)| {
                format!(
                    "# What: invoke {command}; Why: use the installed native runtime.\n{name}{parameters}:\n    @{binary} {command}{forwarded}\n",
                    parameters = if args { " *args" } else { "" },
                    forwarded = if args { " \"$@\"" } else { "" }
                )
            })
            .collect();
        files.push(Justfile {
            path: file,
            source: source.join("\n"),
        });
    }
    files.push(Justfile {
        path: "justfile".into(),
        source: root,
    });
    files
}
