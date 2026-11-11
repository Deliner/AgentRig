use crate::structural_lint::{Diagnostic, Location, Mode, analyze};
use std::{collections::BTreeMap, fs, path::PathBuf, process::Command};
use tempfile::TempDir;

mod attributes;
mod bindings;
mod incomplete;
mod macros;

const SKILL: &str = ".agents/skills/refactor-large-directory/SKILL.md";
const REPAIR: &str = ".agents/skills/repair/SKILL.md";
const FRONTMATTER: &str = "---\nname: fixture\ndescription: Repair the reported finding.\n---\n";
pub const INCOMPLETE: &str = "incomplete";

#[derive(Clone, Copy)]
pub enum Language {
    Rust,
    Python,
}

impl Language {
    fn extension(self) -> &'static str {
        match self {
            Self::Rust => "rs",
            Self::Python => "py",
        }
    }
    fn roots(self) -> &'static str {
        match self {
            Self::Rust => "[src/lib.rs]",
            Self::Python => "[]",
        }
    }
    fn module(self) -> &'static str {
        match self {
            Self::Rust => "mod.rs",
            Self::Python => "api.py",
        }
    }
    fn sources(self) -> [&'static str; 2] {
        match self {
            Self::Rust => [
                "pub fn run() -> i32 { crate::b::value() }\n",
                "pub fn value() -> i32 { 7 }\n",
            ],
            Self::Python => [
                "from b.api import value\ndef run():\n    return value()\n",
                "def value():\n    return 7\n",
            ],
        }
    }
    fn back_edge(self) -> &'static str {
        match self {
            Self::Rust => "pub use crate::a as back;\n",
            Self::Python => "import a.api\n",
        }
    }
}

pub fn config(language: Language) -> String {
    format!(
        "version: 1
config_skill: {REPAIR}
rules:
  - id: architecture
    kind: directory-architecture
    target: directory
    include: [src, 'src/**']
    extensions: [.{}]
    level: error
    architecture:
      python_root: src
      rust_roots: {}
    warning_skill: {SKILL}
    error_skill: {SKILL}
",
        language.extension(),
        language.roots()
    )
}

pub struct Consumer {
    temp: TempDir,
    pub root: PathBuf,
    language: Language,
}

impl Consumer {
    pub fn empty(language: Language) -> Self {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().canonicalize().unwrap().join("consumer");
        let consumer = Self {
            temp,
            root,
            language,
        };
        for skill in [SKILL, REPAIR] {
            consumer.write(skill, FRONTMATTER);
        }
        consumer.write("lint.yaml", &config(language));
        fs::create_dir_all(consumer.root.join("src")).unwrap();
        consumer
    }

    pub fn new(language: Language) -> Self {
        let consumer = Self::empty(language);
        let [source, target] = language.sources();
        consumer.source(source);
        consumer.write(&consumer.file("b"), target);
        if let Language::Rust = language {
            consumer.write("src/lib.rs", "pub mod a; pub mod b;\n");
        }
        for directory in ["src", "src/a", "src/b"] {
            consumer.contract(directory);
        }
        consumer
    }

    fn file(&self, directory: &str) -> String {
        format!("src/{directory}/{}", self.language.module())
    }

    pub fn write(&self, path: &str, text: &str) {
        let path = self.root.join(path);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, text).unwrap();
    }

    pub fn source(&self, text: &str) {
        self.write(&self.file("a"), text);
    }

    pub fn target(&self, text: &str) {
        self.write(&self.file("b"), text);
    }

    pub fn configure(&self, from: &str, to: &str) {
        let path = self.root.join("lint.yaml");
        let text = fs::read_to_string(&path).unwrap();
        assert!(text.contains(from), "{from} is not in {text}");
        fs::write(path, text.replacen(from, to, 1)).unwrap();
    }

    pub fn external(&self, entry: &str) {
        let declaration = format!("python_root: src\n      external:\n        {entry}\n");
        self.configure("python_root: src\n", &declaration);
    }

    pub fn contract(&self, directory: &str) {
        let path = self.root.join(directory);
        let mut files = BTreeMap::new();
        let mut directories = BTreeMap::new();
        for entry in fs::read_dir(&path).unwrap() {
            let entry = entry.unwrap();
            let name = entry.file_name().to_string_lossy().into_owned();
            let directory = entry.file_type().unwrap().is_dir();
            let file = !directory && name != "architecture.yaml";
            if directory {
                directories.insert(name, "Fixture child responsibility");
            } else if file {
                files.insert(name, "Fixture source or resource");
            }
        }
        let name = path.file_name().unwrap().to_string_lossy();
        let inventory = serde_yaml_ng::to_string(&serde_json::json!({
            "files": files,
            "directories": directories,
        }))
        .unwrap();
        let text = format!("purpose: {name} responsibility\n{inventory}");
        fs::write(path.join("architecture.yaml"), text).unwrap();
    }

    pub fn lint(&self) -> Vec<Diagnostic> {
        let config = self.root.join("lint.yaml");
        let location = Location {
            root: &self.root,
            config: &config,
        };
        analyze(location, Mode::Lint).unwrap()
    }

    pub fn passes(&self) {
        let found = self.lint();
        assert!(found.is_empty(), "{:?}", messages(&found));
    }

    pub fn fails(&self, expected: &str) {
        let found = self.lint();
        let blocking = found.iter().any(|item| item.level.blocking());
        let named = found.iter().any(|item| item.message.contains(expected));
        assert!(blocking && named, "{expected}: {:?}", messages(&found));
    }

    pub fn back_edge(&self) {
        let path = self.root.join(self.file("b"));
        let text = fs::read_to_string(&path).unwrap();
        fs::write(path, text + self.language.back_edge()).unwrap();
    }

    pub fn cycles(&self) {
        self.back_edge();
        self.fails("directory dependency cycle");
    }

    pub fn compiles(&self, entry: &str, args: &[&str]) {
        let output = Command::new("rustc")
            .args(["+1.98.1", "--edition=2021", "--crate-type=lib"])
            .args(["--emit=metadata", "--out-dir"])
            .arg(self.temp.path())
            .args(args)
            .arg(self.root.join(entry))
            .output()
            .unwrap();
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(output.status.success(), "{stderr}");
    }
}

pub fn rust(source: &str) -> Consumer {
    let consumer = Consumer::new(Language::Rust);
    consumer.source(source);
    consumer
}

pub fn declared(external: &str, source: &str) -> Consumer {
    let consumer = rust(source);
    consumer.external(&format!("rust: {external}"));
    consumer
}

pub fn messages(found: &[Diagnostic]) -> Vec<&str> {
    found.iter().map(|item| item.message.as_str()).collect()
}
