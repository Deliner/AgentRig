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
pub fn bundle(files: &mut Files, config: &Config) {
    let enabled = config.capabilities.review.is_some();
    if enabled {
        for (path, bytes) in RESOURCES {
            files.insert(
                config.paths.service_path(&format!("review/{path}")),
                bytes.to_vec(),
            );
        }
        files.insert(
            config.paths.service_path("review/.gitignore"),
            b"runtime/\nreports/\n".to_vec(),
        );
    }
}
