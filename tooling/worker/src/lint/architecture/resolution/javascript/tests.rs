use super::*;
use crate::lint::architecture::source;
use std::process::Command;

fn target(path: &str, loader: Loader) -> Target {
    Target::JavaScriptModule {
        path: path.into(),
        loader,
    }
}

fn files(paths: &[&str]) -> BTreeSet<PathBuf> {
    paths.iter().map(PathBuf::from).collect()
}

fn project(inventory: &BTreeSet<PathBuf>) -> tempfile::TempDir {
    let directory = tempfile::tempdir().unwrap();
    for path in inventory {
        let target = directory.path().join(path);
        fs::create_dir_all(target.parent().unwrap()).unwrap();
        fs::write(target, "").unwrap();
    }
    directory
}

fn node(root: &Path, args: &[&str]) -> String {
    let output = Command::new("node")
        .args(args)
        .current_dir(root)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).unwrap()
}

fn resolved(paths: &[&str], path: &str, loader: Loader, mode: Mode) -> Result<Resolved> {
    let inventory = files(paths);
    let external = BTreeSet::new();
    JavaScript::new(Path::new("/"), &inventory, &external, mode)
        .resolve(Path::new(paths[0]), &target(path, loader))
}

#[test]
fn node_import_requires_exact_file_while_require_searches_extensions() {
    let paths = [
        "src/main.js",
        "src/api.js",
        "src/api.json",
        "src/dir/index.js",
    ];
    assert_eq!(
        resolved(&paths, "./api.js", Loader::Import, Mode::Node)
            .unwrap()
            .files,
        files(&[paths[1]])
    );
    assert!(resolved(&paths, "./api", Loader::Import, Mode::Node).is_err());
    assert_eq!(
        resolved(&paths, "./api", Loader::Require, Mode::Node)
            .unwrap()
            .files,
        files(&[paths[1]])
    );
    assert_eq!(
        resolved(&paths, "./dir", Loader::Require, Mode::Node)
            .unwrap()
            .files,
        files(&[paths[3]])
    );
    assert!(resolved(&paths, "./dir", Loader::Import, Mode::Node).is_err());
}

#[test]
fn typescript_substitution_preserves_suffixes_and_priority() {
    let paths = [
        "src/main.ts",
        "src/api.ts",
        "src/api.js",
        "src/view.tsx",
        "src/types.d.ts",
        "src/lib.mts",
        "src/store.cts",
    ];
    for (specifier, expected) in [
        ("./api.js", paths[1]),
        ("./api", paths[1]),
        ("./view", paths[3]),
        ("./types.js", paths[4]),
        ("./lib.mjs", paths[5]),
        ("./store.cjs", paths[6]),
    ] {
        assert_eq!(
            resolved(&paths, specifier, Loader::Import, Mode::TypeScriptBundler)
                .unwrap()
                .files,
            files(&[expected])
        );
    }
    let paths = ["main.ts", "api.js.ts", "lib.mts"];
    assert!(resolved(&paths, "./api.js", Loader::Import, Mode::TypeScriptBundler).is_err());
    assert!(resolved(&paths, "./lib", Loader::Import, Mode::TypeScriptBundler).is_err());
}

#[test]
fn path_normalization_keeps_project_boundary_and_reports_urls() {
    let paths = ["src/app/main.js", "src/api.js"];
    assert_eq!(
        resolved(&paths, "../app/../api.js", Loader::Import, Mode::Node)
            .unwrap()
            .files,
        files(&[paths[1]])
    );
    for path in [
        "../../../outside.js",
        "./api.js?raw",
        "./%2e%2e/api.js",
        "../api.js#part",
        "./a\\b.js",
        "https://example.com/api.js",
    ] {
        assert!(
            resolved(&paths, path, Loader::Import, Mode::Node).is_err(),
            "{path}"
        );
    }
}

#[test]
fn explicit_directory_syntax_does_not_select_a_same_named_file() {
    let paths = ["main.js", "lib.js", "lib/index.js", "empty/index"];
    for specifier in ["./lib/", "./lib/."] {
        assert_eq!(
            resolved(&paths, specifier, Loader::Require, Mode::Node)
                .unwrap()
                .files,
            files(&["lib/index.js"])
        );
        assert!(resolved(&paths, specifier, Loader::Import, Mode::Node).is_err());
    }
    assert!(resolved(&paths, "./empty", Loader::Require, Mode::Node).is_err());
}

#[test]
fn package_names_need_explicit_external_declarations() {
    let inventory = files(&["src/main.js"]);
    let external = BTreeSet::from(["@scope/library".into(), "node:fs".into()]);
    let resolver = JavaScript::new(Path::new("/"), &inventory, &external, Mode::Node);
    for name in ["@scope/library", "@scope/library/api", "node:fs"] {
        let result = resolver
            .resolve(Path::new("src/main.js"), &target(name, Loader::Import))
            .unwrap();
        assert_eq!(result.external.as_deref(), Some(name));
        assert!(result.files.is_empty());
    }
    for name in ["@scope/library-other", "unknown", "#private", "./local.js"] {
        assert!(
            resolver
                .resolve(Path::new("src/main.js"), &target(name, Loader::Import))
                .is_err()
        );
    }
}

#[test]
fn package_entries_are_read_and_invalid_metadata_does_not_disappear() {
    let inventory = files(&[
        "main.js",
        "lib/package.json",
        "lib/run.js",
        "lib/types.d.ts",
        "lib/index.js",
    ]);
    let external = BTreeSet::new();
    let directory = project(&inventory);
    let manifest = directory.path().join("lib/package.json");
    fs::write(&manifest, r#"{"main":"run.js","types":"types.d.ts"}"#).unwrap();
    for (mode, expected) in [
        (Mode::Node, "lib/run.js"),
        (Mode::TypeScriptBundler, "lib/types.d.ts"),
    ] {
        let resolver = JavaScript::new(directory.path(), &inventory, &external, mode);
        assert_eq!(
            resolver
                .resolve(Path::new("main.js"), &target("./lib", Loader::Require))
                .unwrap()
                .files,
            files(&[expected])
        );
    }
    let resolver = JavaScript::new(directory.path(), &inventory, &external, Mode::Node);
    fs::write(&manifest, r#"{"main":"missing.js"}"#).unwrap();
    assert_eq!(
        resolver
            .resolve(Path::new("main.js"), &target("./lib", Loader::Require))
            .unwrap()
            .files,
        files(&["lib/index.js"])
    );
}

#[test]
fn invalid_package_metadata_is_an_error() {
    let inventory = files(&["main.js", "lib/package.json", "lib/index.js"]);
    let directory = project(&inventory);
    let external = BTreeSet::new();
    let manifest = directory.path().join("lib/package.json");
    let resolver = JavaScript::new(directory.path(), &inventory, &external, Mode::Node);
    for contents in ["{", "[]", r#"{"main":5}"#, r#"{"main":"../../escape"}"#] {
        fs::write(&manifest, contents).unwrap();
        assert!(
            resolver
                .resolve(Path::new("main.js"), &target("./lib", Loader::Require))
                .is_err()
        );
    }
}

#[test]
fn unsupported_typescript_metadata_cannot_select_a_fallback_file() {
    let inventory = files(&["main.ts", "lib/package.json", "lib/index.ts"]);
    let directory = project(&inventory);
    let external = BTreeSet::new();
    let resolver = JavaScript::new(
        directory.path(),
        &inventory,
        &external,
        Mode::TypeScriptBundler,
    );
    for contents in [
        r#"{"types":"missing.d.ts"}"#,
        r#"{"typesVersions":{"*":{"*": ["other/*"]}}}"#,
    ] {
        fs::write(directory.path().join("lib/package.json"), contents).unwrap();
        assert!(
            resolver
                .resolve(Path::new("main.ts"), &target("./lib", Loader::Import))
                .is_err()
        );
    }
}

#[test]
fn syntax_to_resolution_preserves_import_require_and_source_lines() {
    let inventory = files(&["app.ts", "api.ts", "other.ts", "store/index.ts"]);
    let external = BTreeSet::new();
    let resolver = JavaScript::new(
        Path::new("/"),
        &inventory,
        &external,
        Mode::TypeScriptBundler,
    );
    let references = source::extract(Path::new("app.ts"), "import type {T} from './api.js';\nexport {x} from './other';\nimport store = require('./store');").unwrap();
    assert!(references.issues.is_empty());
    assert_eq!(
        references
            .items
            .iter()
            .map(|item| item.line)
            .collect::<Vec<_>>(),
        [1, 2, 3]
    );
    let actual: BTreeSet<_> = references
        .items
        .iter()
        .flat_map(|item| {
            resolver
                .resolve(Path::new("app.ts"), &item.target)
                .unwrap()
                .files
        })
        .collect();
    assert_eq!(actual, files(&["api.ts", "other.ts", "store/index.ts"]));
}

#[test]
fn node_resolution_matches_real_require() {
    let inventory = files(&[
        "main.cjs",
        "api.js",
        "api.json",
        "lib.js",
        "lib/index.js",
        "lib/package.json",
        "lib/run.js",
    ]);
    let directory = project(&inventory);
    fs::write(
        directory.path().join("lib/package.json"),
        r#"{"main":"run.js"}"#,
    )
    .unwrap();
    let external = BTreeSet::new();
    let resolver = JavaScript::new(directory.path(), &inventory, &external, Mode::Node);
    for specifier in ["./api", "./api.js", "./lib", "./lib/"] {
        let output = node(
            directory.path(),
            &[
                "-e",
                "console.log(require.resolve(process.argv[1]))",
                specifier,
            ],
        );
        let actual = PathBuf::from(output.trim());
        let expected = resolver
            .resolve(Path::new("main.cjs"), &target(specifier, Loader::Require))
            .unwrap();
        assert_eq!(
            expected.files,
            BTreeSet::from([actual.strip_prefix(directory.path()).unwrap().into()])
        );
    }
}

#[test]
fn node_dynamic_import_requires_the_explicit_file() {
    let directory = project(&files(&["api.js"]));
    node(
        directory.path(),
        &[
            "--input-type=module",
            "-e",
            "await import('./api.js'); try { await import('./api'); process.exit(2); } catch (e) { if (e.code !== 'ERR_MODULE_NOT_FOUND') throw e; }",
        ],
    );
}
