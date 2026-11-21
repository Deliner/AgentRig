// DECISION: D198
use anyhow::Result;
use std::path::Path;

struct Language {
    name: &'static str,
    extensions: &'static [&'static str],
    interpreters: &'static [&'static str],
}
const LANGUAGES: [Language; 16] = [
    Language {
        name: "rust",
        extensions: &["rs"],
        interpreters: &[],
    },
    Language {
        name: "python",
        extensions: &["py", "pyi", "pyw"],
        interpreters: &["python"],
    },
    Language {
        name: "shell",
        extensions: &["sh", "bash", "zsh", "ksh"],
        interpreters: &["sh", "bash", "zsh", "ksh", "dash"],
    },
    Language {
        name: "javascript",
        extensions: &["js", "mjs", "cjs", "jsx"],
        interpreters: &["node"],
    },
    Language {
        name: "typescript",
        extensions: &["ts", "mts", "cts", "tsx"],
        interpreters: &[],
    },
    Language {
        name: "go",
        extensions: &["go"],
        interpreters: &[],
    },
    Language {
        name: "c",
        extensions: &["c", "h"],
        interpreters: &[],
    },
    Language {
        name: "cpp",
        extensions: &["cpp", "cc", "cxx", "hpp", "hh", "hxx"],
        interpreters: &[],
    },
    Language {
        name: "csharp",
        extensions: &["cs"],
        interpreters: &[],
    },
    Language {
        name: "java",
        extensions: &["java"],
        interpreters: &[],
    },
    Language {
        name: "kotlin",
        extensions: &["kt", "kts"],
        interpreters: &[],
    },
    Language {
        name: "swift",
        extensions: &["swift"],
        interpreters: &[],
    },
    Language {
        name: "ruby",
        extensions: &["rb"],
        interpreters: &["ruby"],
    },
    Language {
        name: "perl",
        extensions: &["pl", "pm"],
        interpreters: &["perl"],
    },
    Language {
        name: "php",
        extensions: &["php"],
        interpreters: &["php"],
    },
    Language {
        name: "lua",
        extensions: &["lua"],
        interpreters: &["lua"],
    },
];
pub fn program_languages() -> Vec<&'static str> {
    LANGUAGES.iter().map(|language| language.name).collect()
}
pub fn program_language(
    path: &Path,
    first_line: impl FnOnce() -> Result<Vec<u8>>,
) -> Result<Option<&'static str>> {
    let found = match path.extension() {
        Some(extension) => LANGUAGES
            .iter()
            .find(|language| language.extensions.iter().any(|known| extension == *known)),
        None => interpreter(&first_line()?).and_then(|name| {
            LANGUAGES
                .iter()
                .find(|language| language.interpreters.contains(&name.as_str()))
        }),
    };
    Ok(found.map(|language| language.name))
}
fn interpreter(line: &[u8]) -> Option<String> {
    let command = line.strip_prefix(b"#!")?;
    let command = String::from_utf8_lossy(command);
    let mut words = command.split_whitespace().map(basename);
    let program = words.next();
    let through_env = program == Some("env");
    let launched = if through_env {
        words.find(|word| !word.starts_with('-') && !word.contains('='))
    } else {
        program
    };
    launched.map(|name| {
        name.trim_end_matches(|character: char| character.is_ascii_digit() || character == '.')
            .to_owned()
    })
}
fn basename(word: &str) -> &str {
    word.rsplit_once('/').map_or(word, |(_, name)| name)
}
