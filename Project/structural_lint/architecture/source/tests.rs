use super::{Reference, References, Target, extract};
use std::path::Path;

fn parse(path: &str, text: &str) -> References {
    extract(Path::new(path), text).unwrap()
}

fn targets(result: &References) -> Vec<Target> {
    assert!(result.issues.is_empty(), "{:?}", result.issues);
    result
        .items
        .iter()
        .map(|item| item.target.clone())
        .collect()
}

fn rust_path(path: &str) -> Target {
    Target::RustPath {
        path: path.into(),
        scope: Vec::new(),
    }
}

#[test]
fn rust_use_groups_aliases_wildcards_and_qualified_calls() {
    let source = "use crate::domain::{self, api::Item as Renamed, private::*};\nfn f() { crate::storage::save(); }";
    let result = parse("src/lib.rs", source);
    assert_eq!(
        targets(&result),
        [
            rust_path("crate::domain"),
            rust_path("crate::domain::api::Item"),
            rust_path("crate::domain::private"),
            rust_path("crate::storage::save"),
        ]
    );
    assert_eq!(result.items.last().unwrap().line, 2);
}

#[test]
fn rust_inline_scopes_and_module_declarations_retain_resolution_context() {
    let source = "mod app { mod nested; use super::domain::Api; }";
    assert_eq!(
        targets(&parse("src/lib.rs", source)),
        [
            Target::RustModule {
                path: vec!["app".into()],
                inline: true,
                file: None,
            },
            Target::RustModule {
                path: vec!["app".into(), "nested".into()],
                inline: false,
                file: None,
            },
            Target::RustPath {
                path: "super::domain::Api".into(),
                scope: vec!["app".into()]
            },
        ]
    );
}

#[test]
fn rust_comments_raw_identifiers_and_generic_arguments_preserve_paths() {
    let source = "use crate /* note */ :: r#type::{ /* item */ * };\nfn f() { Vec::<crate::model::Item>::new(); }";
    let result = parse("src/lib.rs", source);
    let found = targets(&result);
    assert!(found.contains(&rust_path("crate::type")), "{found:?}");
    assert!(
        found.contains(&rust_path("crate::model::Item")),
        "{found:?}"
    );
    assert!(
        found
            .iter()
            .all(|target| !format!("{target:?}").contains("note"))
    );
}

#[test]
fn python_imports_keep_original_names_relative_depth_and_members() {
    let source = "import domain.api as api, storage\nfrom ..domain import api as service, data\nfrom . import helper\nfrom domain.api import *\n";
    assert_eq!(
        targets(&parse("src/app/main.py", source)),
        [
            Target::PythonModule("domain.api".into()),
            Target::PythonModule("storage".into()),
            Target::PythonFrom {
                module: "..domain".into(),
                names: vec!["api".into(), "data".into()]
            },
            Target::PythonFrom {
                module: ".".into(),
                names: vec!["helper".into()]
            },
            Target::PythonFrom {
                module: "domain.api".into(),
                names: vec!["*".into()]
            },
        ]
    );
}

#[test]
fn python_static_loading_is_extracted_and_dynamic_loading_is_incomplete() {
    let result = parse(
        "app.py",
        "__import__('domain.api')\nimportlib.import_module(name)\n",
    );
    assert_eq!(
        result.items,
        [Reference {
            line: 1,
            target: Target::PythonModule("domain.api".into())
        }]
    );
    assert_eq!(result.issues.len(), 1);
    assert_eq!(result.issues[0].line, Some(2));
}

#[test]
fn python_static_string_forms_preserve_module_values() {
    let source = "__import__(r'domain.api')\n__import__('''storage.api''')\n__import__(f'{name}')";
    let result = parse("app.py", source);
    assert_eq!(
        result
            .items
            .iter()
            .map(|item| item.target.clone())
            .collect::<Vec<_>>(),
        [
            Target::PythonModule("domain.api".into()),
            Target::PythonModule("storage.api".into()),
        ]
    );
    assert_eq!(result.issues.len(), 1);
    assert_eq!(result.issues[0].line, Some(3));
}

#[test]
fn strings_and_comments_are_not_dependencies() {
    for (path, source) in [
        (
            "a.rs",
            "// use crate::fake;\nconst S: &str = \"crate::fake::run()\";",
        ),
        ("a.py", "# import fake\nx = 'from fake import value'"),
    ] {
        assert!(targets(&parse(path, source)).is_empty(), "{path}");
    }
}

#[test]
fn malformed_syntax_never_returns_successful_empty_analysis() {
    for (path, source) in [("a.rs", "use crate::{;"), ("a.py", "from . import (")] {
        let result = parse(path, source);
        assert!(result.items.is_empty(), "{path}");
        assert_eq!(result.issues.len(), 1, "{path}");
        assert!(result.issues[0].message.contains("malformed"));
    }
    assert!(extract(Path::new("app.go"), "package app").is_err());
}

// DECISION: D136
#[test]
fn rust_expansion_forms_become_includes_macro_paths_and_generated_scopes() {
    let source = "include!(\"part.rs\");\nmod out { include!(concat!(env!(\"OUT_DIR\"), \"/gen.rs\")); }\nmacro_rules! make { () => { $crate::b::value(); Self::new() } }\nmake!();\n#[unsafe(no_mangle)] fn f() { use crate::c as d; d::run(); }\n#[custom::attr] fn g() {}";
    let result = parse("src/lib.rs", source);
    let found = targets(&result);
    let include = Target::RustInclude {
        file: "part.rs".into(),
        scope: Vec::new(),
    };
    assert_eq!(found.first(), Some(&include), "{found:?}");
    assert!(found.contains(&rust_path("crate::b::value")), "{found:?}");
    assert!(found.contains(&rust_path("crate::c::run")), "{found:?}");
    assert!(
        found
            .iter()
            .all(|target| !format!("{target:?}").contains("Self"))
    );
    let attributes: Vec<_> = found
        .iter()
        .filter_map(|target| match target {
            Target::RustMacro(call)
                if call.path.contains("attr") || call.path.contains("mangle") =>
            {
                Some(call.path.as_str())
            }
            _ => None,
        })
        .collect();
    assert_eq!(attributes, ["custom::attr"]);
    let macros: Vec<_> = result
        .rust_macros
        .iter()
        .map(|item| item.name.as_str())
        .collect();
    assert_eq!(macros, ["make"]);
    assert_eq!(result.rust_generated, [vec!["out".to_owned()], Vec::new()]);
}

#[test]
fn unsupported_dependency_forms_are_located_and_reported() {
    for (path, source) in [
        ("a.py", "importlib.import_module('.x', package)"),
        ("a.rs", "include!(concat!(\"module\", \".rs\"));"),
        ("a.rs", "mod inline { #[path = \"other.rs\"] mod a; }"),
        ("a.rs", "macro_rules! make { () => { use crate::hidden; } }"),
    ] {
        let result = parse(path, source);
        assert!(!result.issues.is_empty(), "{path}: {source}");
        assert!(result.issues.iter().all(|issue| issue.line == Some(1)));
    }
}
