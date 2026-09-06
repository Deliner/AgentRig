use super::{Config, Files};
const RESOURCES: &[(&str, &[u8])] = &[
    (
        "config/contracts/code.json",
        include_bytes!("../../../review/config/contracts/code.json"),
    ),
    (
        "config/contracts/research.json",
        include_bytes!("../../../review/config/contracts/research.json"),
    ),
    (
        "config/projects/code.yaml",
        include_bytes!("../../../review/config/projects/code.yaml"),
    ),
    (
        "config/projects/research.yaml",
        include_bytes!("../../../review/config/projects/research.yaml"),
    ),
    (
        "config/review.yaml",
        include_bytes!("../../../review/config/review.yaml"),
    ),
    (
        "prompts/correctness.md",
        include_bytes!("../../../review/prompts/correctness.md"),
    ),
    (
        "prompts/methodology.md",
        include_bytes!("../../../review/prompts/methodology.md"),
    ),
    (
        "prompts/requirements.md",
        include_bytes!("../../../review/prompts/requirements.md"),
    ),
];
pub fn bundle(files: &mut Files, config: &Config) -> anyhow::Result<()> {
    let enabled = config.capabilities.review.is_some();
    if enabled {
        for (path, bytes) in RESOURCES {
            let selected_project = path.starts_with("config/projects/")
                && config.vcs.backend != review_runner::vcs::Kind::Git;
            let contents = if selected_project {
                let mut project: review_runner::config::Project =
                    review_runner::config::yaml::decode(std::str::from_utf8(bytes)?)?;
                project.repository.vcs = config.vcs.backend;
                review_runner::config::yaml::encode(&project)?.into_bytes()
            } else {
                bytes.to_vec()
            };
            files.insert(
                config.paths.service_path(&format!("review/{path}")),
                contents,
            );
        }
        files.insert(
            config.paths.service_path("review/.gitignore"),
            b"runtime/\nreports/\n".to_vec(),
        );
    }
    Ok(())
}
