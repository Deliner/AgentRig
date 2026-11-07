use anyhow::{Context, Result};
use std::path::Path;
use tree_sitter::{Language, Parser, Tree};

pub fn parse(path: &Path, source: &str) -> Result<Option<Tree>> {
    let language: Language = match path.extension().and_then(|extension| extension.to_str()) {
        // DECISION: D112
        Some("rs") => tree_sitter_rust::LANGUAGE.into(),
        Some("py" | "pyi") => tree_sitter_python::LANGUAGE.into(),
        Some("sh") => tree_sitter_bash::LANGUAGE.into(),
        _ => return Ok(None),
    };
    let mut parser = Parser::new();
    parser.set_language(&language)?;
    Ok(Some(
        parser
            .parse(source, None)
            .context("parser returned no tree")?,
    ))
}
// DECISION: D037
pub fn reference(comment: &str) -> Option<&str> {
    let body = comment.trim_end();
    let marked = body.strip_prefix("//").or_else(|| body.strip_prefix('#'))?;
    let (label, id) = marked.strip_prefix(' ')?.split_once(": ")?;
    let prefix = match label {
        "DECISION" => "D",
        "INVARIANT" => "I",
        _ => return None,
    };
    let digits = id.strip_prefix(prefix)?;
    let numbered = digits.len() == 3 && digits.bytes().all(|byte| byte.is_ascii_digit());
    numbered.then_some(id)
}
// DECISION: D067
struct Token {
    name: &'static str,
    prefix: &'static str,
    length: usize,
    class: fn(char) -> bool,
}
impl Token {
    fn find(&self, literal: &str) -> Option<&'static str> {
        literal.match_indices(self.prefix).find_map(|(index, _)| {
            let tail = &literal[index + self.prefix.len()..];
            let run = tail
                .chars()
                .take_while(|character| (self.class)(*character))
                .count();
            (run >= self.length).then_some(self.name)
        })
    }
}
fn upper_digit(character: char) -> bool {
    character.is_ascii_uppercase() || character.is_ascii_digit()
}
fn base62(character: char) -> bool {
    character.is_ascii_alphanumeric()
}
fn url_safe(character: char) -> bool {
    character.is_ascii_alphanumeric() || character == '_' || character == '-'
}
const TOKENS: &[Token] = &[
    Token {
        name: "aws-access-key",
        prefix: "AKIA",
        length: 16,
        class: upper_digit,
    },
    Token {
        name: "github-token",
        prefix: "ghp_",
        length: 36,
        class: base62,
    },
    Token {
        name: "github-token",
        prefix: "gho_",
        length: 36,
        class: base62,
    },
    Token {
        name: "github-token",
        prefix: "ghu_",
        length: 36,
        class: base62,
    },
    Token {
        name: "github-token",
        prefix: "ghs_",
        length: 36,
        class: base62,
    },
    Token {
        name: "github-token",
        prefix: "ghr_",
        length: 36,
        class: base62,
    },
    Token {
        name: "github-token",
        prefix: "github_pat_",
        length: 82,
        class: url_safe,
    },
    Token {
        name: "google-api-key",
        prefix: "AIza",
        length: 35,
        class: url_safe,
    },
    Token {
        name: "slack-token",
        prefix: "xoxb-",
        length: 10,
        class: url_safe,
    },
    Token {
        name: "slack-token",
        prefix: "xoxp-",
        length: 10,
        class: url_safe,
    },
    Token {
        name: "slack-token",
        prefix: "xoxa-",
        length: 10,
        class: url_safe,
    },
    Token {
        name: "slack-token",
        prefix: "xoxr-",
        length: 10,
        class: url_safe,
    },
];
pub fn secret(literal: &str) -> Option<&'static str> {
    let private_key = literal.contains("-----BEGIN") && literal.contains("PRIVATE KEY");
    if private_key {
        return Some("private-key");
    }
    TOKENS.iter().find_map(|token| token.find(literal))
}
