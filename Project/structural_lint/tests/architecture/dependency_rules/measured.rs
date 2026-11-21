use crate::structural_lint::tests::architecture::{Consumer, FRONTMATTER, Language};
use std::{collections::BTreeSet, path::Path};

pub fn measured(
    language: Language,
    rule: &str,
    files: &[(impl AsRef<str>, impl AsRef<str>)],
) -> Consumer {
    let consumer = Consumer::empty(language);
    let skill = format!(".agents/skills/{rule}/SKILL.md");
    consumer.write(&skill, FRONTMATTER);
    let config = format!(
        "rules:
  - id: {rule}
    kind: {rule}
    target: directory
    include: [src, 'src/**']
    level: error
    warning_skill: {skill}
    error_skill: {skill}
"
    );
    consumer.configure("rules:\n", &config);
    let mut directories = BTreeSet::new();
    for (path, text) in files {
        consumer.write(path.as_ref(), text.as_ref());
        let parents = Path::new(path.as_ref()).ancestors().skip(1);
        let owned = parents.filter(|parent| parent.starts_with("src"));
        directories.extend(owned.map(Path::to_path_buf));
    }
    for directory in directories {
        consumer.contract(directory.to_str().unwrap());
    }
    consumer
}
