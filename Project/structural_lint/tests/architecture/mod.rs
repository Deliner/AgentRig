use crate::shared::testing::{launch, run, worker};
use crate::structural_lint::rules::{Kind, structure::Structure};
use crate::structural_lint::{Diagnostic, Location, Mode, analyze, catalog};
use serde_json::Value;
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    process::Command,
};
use tempfile::TempDir;

mod attributes;
mod bindings;
mod cli;
mod coupling;
mod crates;
mod disallowed;
mod externals;
mod incomplete;
mod inventory;
mod macros;
mod resources;
mod settings;
mod versioned;
mod wiring;

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

    fn analysis(&self, mode: Mode) -> anyhow::Result<Vec<Diagnostic>> {
        let config = self.root.join("lint.yaml");
        let location = Location {
            root: &self.root,
            config: &config,
        };
        analyze(location, mode)
    }

    pub fn lint(&self) -> Vec<Diagnostic> {
        self.analysis(Mode::Lint).unwrap()
    }

    pub fn located(&self) -> Vec<(String, Option<usize>, String)> {
        let found = self.lint();
        let blocking = found.iter().all(|item| item.level.blocking());
        assert!(blocking, "{:?}", messages(&found));
        let location = |item: Diagnostic| (item.path, item.line, item.message);
        found.into_iter().map(location).collect()
    }

    pub fn process(&self, binary: &Path) -> (Option<i32>, Value) {
        let args = ["lint", "--config", "lint.yaml", "--json", "--warnings"];
        let result = run(&mut launch(binary, &self.root, &args), "");
        let parsed = serde_json::from_str(&result.stdout);
        let mut findings: Value =
            parsed.unwrap_or_else(|error| panic!("{error}: {}", result.output()));
        for item in findings.as_array_mut().unwrap() {
            let rerun = item.as_object_mut().unwrap().remove("rerun").unwrap();
            let named = rerun.as_str().unwrap().contains(binary.to_str().unwrap());
            assert!(named, "{rerun}");
        }
        (result.code, findings)
    }

    pub fn invalid(&self) {
        let checked = self.analysis(Mode::ConfigCheck);
        assert!(checked.is_err(), "the configuration check passed");
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

    pub fn rejects(&self, expected: &str) {
        match self.analysis(Mode::Lint) {
            Ok(_) => self.fails(expected),
            Err(error) => assert!(format!("{error:#}").contains(expected), "{error:#}"),
        }
    }

    pub fn snapshot(&self) -> BTreeMap<PathBuf, Vec<u8>> {
        let mut files = BTreeMap::new();
        let mut pending = vec![self.root.clone()];
        while let Some(directory) = pending.pop() {
            for entry in fs::read_dir(directory).unwrap() {
                let path = entry.unwrap().path();
                let nested = path.is_dir();
                if nested {
                    pending.push(path);
                } else {
                    files.insert(path.clone(), fs::read(path).unwrap());
                }
            }
        }
        files
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
            .arg("-L")
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

pub fn described() -> Value {
    catalog::describe(Kind::Structure(Structure::DirectoryArchitecture))
}

pub fn binaries() -> [PathBuf; 2] {
    [
        worker().to_owned(),
        worker().with_file_name("agentrig-lint"),
    ]
}
